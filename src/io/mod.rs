use std::sync::mpsc;
use std::io;
use std::collections::VecDeque;
use std::time::Duration;



pub enum Message {
    Bytes(Vec<u8>),
    Eof,
}


pub struct IoSender {
    sender: mpsc::Sender<Message>,
}

impl IoSender {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }
}

impl io::Write for IoSender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sender.send(Message::Bytes(Vec::from(buf))).map_err(|err| io::Error::new(io::ErrorKind::NotConnected, err))?;
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> io::Result<()>{
        self.sender.send(Message::Eof).map_err(|err| io::Error::new(io::ErrorKind::NotConnected, err))
    }
}


pub struct IoReceiver {
    receiver: mpsc::Receiver<Message>,
    buffer: VecDeque<u8>,
    eof: bool,
    timeout: Duration,
}

impl IoReceiver {
    pub fn new(receiver: mpsc::Receiver<Message>, timeout: Duration) -> Self {
        Self {
            receiver,
            timeout,
            buffer: VecDeque::new(),
            eof: false,
        }
    }
}

impl io::Read for IoReceiver {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.eof && self.buffer.len() == 0 {
            self.eof = false;
            return Ok(0);
        }
        
        let msg = self.receiver.recv_timeout(self.timeout).map_err(|err| {
            if let mpsc::RecvTimeoutError::Timeout = err {
                io::Error::new(io::ErrorKind::TimedOut, err)
            }
            else {
                io::Error::new(io::ErrorKind::NotConnected, err)
            }
        })?;

        match msg {
            Message::Bytes(bytes) => {
                self.buffer.extend(bytes.iter());
                self.eof = false;
            },
            Message::Eof => {
                self.eof = true;
            },
        }

        let n = buf.len().min(self.buffer.len());
        let read_bytes = self.buffer.drain(0..n).collect::<Vec<u8>>();
        buf[0..n].copy_from_slice(&read_bytes);

        Ok(n)
    }
}


pub struct OutputBuffer {
    data: Vec<u8>,
    start: usize,
    capacity: usize,
}

impl OutputBuffer {
    pub fn new(capacity: usize) -> Self {
        Self{ data: Vec::with_capacity(capacity), start: 0, capacity }
    }

    pub fn to_string(&self) -> Result<String, std::string::FromUtf8Error> {
        if self.data.len() < self.capacity {
            String::from_utf8(self.data.clone())
        }
        else {
            let first = &self.data[self.start..];
            let second = &self.data[..self.start];

            String::from_utf8([first, second].concat())
        }
    }

    pub fn is_truncated(&self) -> bool {
        self.data.len() == self.capacity
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.start = 0;
    }
}

impl std::io::Write for OutputBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.data.len() < self.capacity {
            let remaining_space = self.capacity - self.data.len();
            if buf.len() < remaining_space {
                self.data.extend_from_slice(buf);
            }
            else {
                let (first, second) = buf.split_at(remaining_space);
                self.data.extend_from_slice(first);
                self.write(second)?;
            }
        }
        else {
            let remaining_space = self.capacity - self.start;
            if buf.len() < remaining_space {
                self.data[self.start .. (self.start + buf.len())].copy_from_slice(buf);
                self.start += buf.len();
            }
            else {
                //               4            6
                //             ----        ------
                // data: 0123456789   buf: abcdef
                //             ^           012345
                //
                //               4             4             2
                //             ----          ----           --
                // data: 0123456789   first: abcd   second: ef
                //             ^             0123           
                //
                // data: ef2345abcd
                //         ^
                let (first, second) = buf.split_at(remaining_space);
                self.data[self.start..].copy_from_slice(first);
                self.data[..second.len()].copy_from_slice(second);
                self.start = second.len();
            }
        }
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()>{
        Ok(())
    }
}


#[cfg(test)]
mod tests;

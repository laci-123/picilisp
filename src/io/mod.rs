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
    timeout: Duration,
}

impl IoReceiver {
    pub fn new(receiver: mpsc::Receiver<Message>, timeout: Duration) -> Self {
        Self {
            receiver,
            timeout,
            buffer: VecDeque::new(),
        }
    }
}

impl io::Read for IoReceiver {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buffer.len() == 0 {
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
                },
                Message::Eof => {
                    return Ok(0);
                },
            }
        }

        let n = buf.len().min(self.buffer.len());
        let read_bytes = self.buffer.drain(0..n).collect::<Vec<u8>>();
        buf[0..n].copy_from_slice(&read_bytes);

        Ok(n)
    }
}


pub fn make_io(timeout: Duration) -> (IoSender, IoReceiver) {
    let (s, r) = mpsc::channel();
    (IoSender::new(s), IoReceiver::new(r, timeout))
}



#[cfg(test)]
mod tests;

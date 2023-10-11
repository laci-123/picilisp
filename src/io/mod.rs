#[derive(Debug)]
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

pub struct BytesBuffer {
    bytes: Vec<u8>,
    cursor: usize,
}

impl BytesBuffer {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            cursor: 0
        }
    }

    pub fn empty() -> Self {
        Self::from_bytes(vec![])
    }

    pub fn write_bytes(&mut self, new_bytes: &[u8]) {
        self.bytes.extend_from_slice(new_bytes);
    }

    pub fn read_bytes(&mut self, bytes_count: usize) -> Option<&[u8]> {
        if self.bytes.len() - self.cursor < bytes_count {
            None
        }
        else {
            let old_cursor = self.cursor;
            self.cursor += bytes_count;

            Some(&self.bytes[old_cursor..self.cursor])
        }
    }

    pub fn read_all(&mut self) -> Option<&[u8]> {
        self.read_bytes(self.bytes.len() - self.cursor)
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn testing_bytes_buffer() {
        let bytes = b"Hello World";
        let mut bytes_buffer = BytesBuffer::from_bytes(Vec::from(bytes));

        let read_bytes = bytes_buffer.read_bytes(2).unwrap();
        let str1 = String::from_utf8_lossy(read_bytes);
        println!("str1 = {str1}");

        let read_bytes2 = bytes_buffer.read_all().unwrap();
        let str2 = String::from_utf8_lossy(read_bytes2);
        println!("str2 = {str2}");

        let b = bytes_buffer.read_bytes(5);
        println!("Should be None {b:?}");
    }
}
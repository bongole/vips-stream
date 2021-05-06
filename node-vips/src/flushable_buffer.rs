pub struct FlushableBuffer {
    buffer: Vec<u8>,
    high_water_mark: usize,
}

impl FlushableBuffer {
    pub fn new(hwm: Option<usize>) -> Self {
        Self {
            buffer: Vec::with_capacity(hwm.unwrap_or(16 * 1024)),
            high_water_mark: hwm.unwrap_or(16 * 1024),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> bool {
        self.buffer.extend_from_slice(buf);
        self.buffer.len() < self.high_water_mark
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn flush<F>(&mut self, mut f: F)
    where
        F: FnMut(&[u8]),
    {
        f(&self.buffer[..]);
        self.buffer.clear();
    }
}

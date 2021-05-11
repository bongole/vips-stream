pub struct FlushableBuffer {
    buffer: Vec<u8>,
    high_water_mark: usize,
    closed: bool,
}

impl FlushableBuffer {
    pub fn new(hwm: Option<usize>) -> Self {
        Self {
            buffer: Vec::with_capacity(hwm.unwrap_or(16 * 1024)),
            high_water_mark: hwm.unwrap_or(16 * 1024),
            closed: false,
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> bool {
        self.buffer.extend(buf);
        self.buffer.len() < self.high_water_mark
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn flush<F>(&mut self, mut f: F)
    where
        F: FnMut(&[u8]),
    {
        f(&self.buffer[..]);
        self.buffer.clear();
    }
}

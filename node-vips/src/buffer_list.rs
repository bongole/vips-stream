use std::{cmp::Ordering, collections::VecDeque, ops::Deref};

pub struct BufferList<T> {
    buf_list: VecDeque<T>,
    garbage_list: VecDeque<T>,
    pos_from_head: usize,
    total_len: usize,
    high_water_mark: usize,
    closed: bool
}
#[derive(Debug, Clone)]
pub enum ReadError {
    Error,
    NeedMoreChunk,
}

impl<T: Deref<Target = [u8]> + AsRef<[u8]>> BufferList<T> {
    pub fn new(hwm: Option<usize>) -> Self {
        Self {
            buf_list: VecDeque::new(),
            garbage_list: VecDeque::new(),
            pos_from_head: 0,
            total_len: 0,
            high_water_mark: hwm.unwrap_or(128 * 1024),
            closed: false
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn len(&self) -> usize {
        self.total_len
    }

    pub fn is_empty(&self) -> bool {
        let is_empty = self.buf_list.is_empty();
        if is_empty {
            debug_assert_eq!(0, self.total_len);
        }

        is_empty
    }

    pub fn gc<F>(&mut self, mut f: F)
    where
        F: FnMut(T),
    {
        while !self.garbage_list.is_empty() {
            f(self.garbage_list.pop_front().unwrap());
        }
    }

    pub fn push(&mut self, chunk: T) -> bool {
        if self.closed {
            return false;
        }

        self.total_len += chunk.len();
        self.buf_list.push_back(chunk);

        self.total_len < self.high_water_mark
    }

    pub fn read(&mut self, read_buf: &mut [u8]) -> Result<usize, ReadError> {
        if self.is_empty() && self.closed {
            return Err(ReadError::Error);
        }

        if self.is_empty() && !read_buf.is_empty() {
            return Err(ReadError::NeedMoreChunk);
        }

        let read_buf_len = read_buf.len();
        match read_buf_len.cmp(&self.total_len) {
            Ordering::Less => {
                let mut read_buf_pos = 0;
                loop {
                    let b = &self.buf_list[0];
                    let b_ref = &b.as_ref()[self.pos_from_head..];
                    let r_ref = &mut read_buf[read_buf_pos..];

                    match r_ref.len().cmp(&b_ref.len()) {
                        Ordering::Equal => {
                            r_ref.copy_from_slice(b_ref);
                            self.pos_from_head = 0;
                            self.garbage_list.push_back(self.buf_list.pop_front().unwrap());
                            break;
                        }
                        Ordering::Less => {
                            r_ref.copy_from_slice(&b_ref[..r_ref.len()]);
                            self.pos_from_head += r_ref.len();
                            break;
                        }
                        Ordering::Greater => {
                            r_ref[..b_ref.len()].copy_from_slice(b_ref);
                            self.pos_from_head = 0;
                            read_buf_pos += b_ref.len();
                            self.garbage_list.push_back(self.buf_list.pop_front().unwrap());
                        }
                    }
                }

                self.total_len -= read_buf_len;
                Ok(read_buf_len)
            }
            Ordering::Equal => {
                let mut read_buf_pos = 0;
                while !self.buf_list.is_empty() {
                    let b = self.buf_list.pop_front().unwrap();
                    let b_ref = &b.as_ref()[self.pos_from_head..];
                    read_buf[read_buf_pos..b_ref.len()].copy_from_slice(b_ref);
                    self.pos_from_head = 0;
                    read_buf_pos += b_ref.len();
                    self.garbage_list.push_back(b);
                }

                self.total_len -= read_buf_len;
                Ok(read_buf_len)
            }
            Ordering::Greater => {
                let mut read_buf_pos = 0;
                while !self.buf_list.is_empty() {
                    let b = self.buf_list.pop_front().unwrap();
                    let b_ref = &b.as_ref()[self.pos_from_head..];
                    read_buf[read_buf_pos..b_ref.len()].copy_from_slice(b_ref);
                    self.pos_from_head = 0;
                    read_buf_pos += b_ref.len();
                    self.garbage_list.push_back(b);
                }

                let r = self.total_len;
                self.total_len = 0;
                Ok(r)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer_list::ReadError;

    use super::BufferList;

    #[test]
    fn test_len() {
        let mut bl = BufferList::new(None);
        let v1 = vec![1u8, 2u8, 3u8];
        let v2 = vec![4u8, 5u8, 6u8];
        bl.push(v1);
        bl.push(v2);

        assert_eq!(6, bl.len());
    }

    #[test]
    fn test_empty() {
        let mut bl = BufferList::new(None);
        assert_eq!(true, bl.is_empty());

        let v1 = vec![1u8, 2u8, 3u8];
        let v2 = vec![4u8, 5u8, 6u8];
        bl.push(v1);
        bl.push(v2);

        assert_eq!(false, bl.is_empty());
    }

    #[test]
    fn test_read_empty() {
        let mut bl: BufferList<Vec<u8>> = BufferList::new(None);

        let mut a1 = [0_u8; 3];
        let r = bl.read(&mut a1);
        assert!(matches!(r, Err(ReadError::NeedMoreChunk)));

        let mut a2 = [0u8; 0];
        let r = bl.read(&mut a2).unwrap();
        assert_eq!(0, r);
    }

    #[test]
    fn test_read_eq() {
        let mut bl = BufferList::new(None);

        let v1 = vec![1u8, 2u8, 3u8];
        bl.push(v1);

        let mut a1 = [0_u8; 3];
        let r = bl.read(&mut a1).unwrap();

        assert_eq!(3, r);
        assert_eq!(0, bl.len());
        assert_eq!(true, bl.is_empty());
        assert_eq!([1u8, 2u8, 3u8], a1);
    }

    #[test]
    fn test_read_gt() {
        let mut bl = BufferList::new(None);

        let v1 = vec![1u8, 2u8, 3u8];
        bl.push(v1);

        let mut a1 = [0_u8; 4];
        let r = bl.read(&mut a1).unwrap();

        assert_eq!(3, r);
        assert_eq!(0, bl.len());
        assert_eq!(true, bl.is_empty());
        assert_eq!([1u8, 2u8, 3u8, 0u8], a1);
    }

    #[test]
    fn test_read_lt() {
        let mut bl = BufferList::new(None);

        let v1 = vec![1u8, 2u8, 3u8];
        bl.push(v1);

        let mut a1 = [0_u8; 2];
        let r = bl.read(&mut a1).unwrap();

        assert_eq!(2, r);
        assert_eq!(1, bl.len());
        assert_eq!(false, bl.is_empty());
        assert_eq!([1u8, 2u8], a1);
    }

    #[test]
    fn test_close() {
        let mut bl = BufferList::new(Some(1024));

        let v1 = vec![1u8, 2u8, 3u8];
        let r = bl.push(v1);
        assert_eq!(true, r);

        let v2 = vec![4u8, 5u8, 6u8];
        bl.close();
        let r = bl.push(v2);
        assert_eq!(false, r);

        let mut a1 = [0_u8; 4];
        let r = bl.read(&mut a1).unwrap();

        assert_eq!(3, r);
        assert_eq!([1u8, 2u8, 3u8, 0u8], a1);

        let mut a2 = [0_u8; 4];
        let r = bl.read(&mut a2);

        assert_eq!(true, bl.is_empty());
        assert!(matches!(r, Err(ReadError::Error)));
        assert_eq!([0u8, 0u8, 0u8, 0u8], a2);

    }

    #[test]
    fn test_high_water_mark() {
        let mut bl = BufferList::new(Some(5));

        let v1 = vec![1u8, 2u8, 3u8];
        let r = bl.push(v1);
        assert_eq!(true, r);

        let v2 = vec![1u8, 2u8, 3u8];
        let r = bl.push(v2);
        assert_eq!(false, r);
    }

    #[test]
    fn test_buffers() {
        let mut bl = BufferList::new(None);

        let v1 = vec![1u8, 2u8, 3u8];
        let v2 = vec![4u8, 5u8, 6u8, 7u8];
        bl.push(v1);
        bl.push(v2);

        let mut a1 = [0_u8; 2];
        bl.read(&mut a1).unwrap();
        assert_eq!(5, bl.len());
        assert_eq!([1u8, 2u8], a1);

        let mut a2 = [0_u8; 3];
        bl.read(&mut a2).unwrap();
        assert_eq!(2, bl.len());
        assert_eq!([3u8, 4u8, 5u8], a2);

        let mut a3 = [0_u8; 3];
        let r = bl.read(&mut a3).unwrap();
        assert_eq!(2, r);
        assert_eq!(0, bl.len());
        assert_eq!([6u8, 7u8, 0u8], a3);
    }

    #[test]
    fn test_gc() {
        let mut bl = BufferList::new(Some(5));

        let v1 = vec![1u8, 2u8, 3u8];
        let v2 = vec![1u8, 2u8, 3u8];
        bl.push(v1);
        bl.push(v2);

        let mut a1 = [0u8; 3];
        bl.read(&mut a1).unwrap();
        let mut gc_count = 0;
        bl.gc(|_a| {
            gc_count += 1;
        });

        assert_eq!(1, gc_count);
        assert_eq!(3, bl.len());
    }
}

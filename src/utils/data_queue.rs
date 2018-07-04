use bytes::Bytes;
use std::cmp::{self, Ordering, PartialOrd};
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;
use utils::RevOrd;

#[derive(Debug, PartialEq, Eq)]
struct DataChunk {
    offset: usize,
    bytes: Bytes,
}

impl PartialOrd for DataChunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataChunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

impl DataChunk {
    pub fn new(offset: usize, bytes: Bytes) -> Self {
        Self { offset, bytes }
    }

    pub fn try_advance(&mut self, count: usize) -> usize {
        let to_advance = cmp::min(count, self.bytes.len());

        self.bytes.advance(to_advance);
        self.offset += to_advance;

        to_advance
    }

    /// # Panics
    ///
    /// If there are not enough bytes to advance by `count`.
    pub fn advance(&mut self, count: usize) {
        assert_eq!(self.try_advance(count), count);
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn end_offset(&self) -> usize {
        self.offset + self.bytes.len()
    }
}

/// This is for a queue of data where each chunk of data may be inserted out of order.
#[derive(Debug, Default)]
pub struct DataQueue {
    pending_chunks: BinaryHeap<RevOrd<DataChunk>>,
    read_offset: usize,
    last_offset: Option<usize>,
}

impl DataQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_finished(&self) -> bool {
        self.last_offset
            .map(|l| l == self.read_offset)
            .unwrap_or(false)
    }

    pub fn insert_chunk(&mut self, offset: usize, last: bool, bytes: Bytes) {
        let data_chunk = DataChunk::new(offset, bytes);

        let end_offset = data_chunk.end_offset();
        if last {
            if let Some(last_offset) = self.last_offset {
                assert_eq!(
                    end_offset, last_offset,
                    "once the last offset has been set it cannot be moved"
                );
            } else {
                assert!(
                    end_offset >= self.read_offset,
                    "the last offset cannot be before the bytes we have currently read until"
                );
                self.last_offset = Some(end_offset);
            }
        } else if end_offset <= self.read_offset {
            // don't bother inserting the chunk if the bytes have already been read past
            return;
        }

        self.pending_chunks.push(RevOrd(data_chunk));
    }

    pub fn read(&mut self, mut buf: &mut [u8]) -> usize {
        let mut read_bytes = 0;

        // while there are more bytes to fill
        while !buf.is_empty() {
            match self.pending_chunks.peek_mut() {
                None => break,
                Some(mut current_chunk) => {
                    match current_chunk.offset.cmp(&self.read_offset) {
                        Ordering::Greater => {
                            debug!(
                                "the bytes starting at offset {} are not yet available",
                                self.read_offset
                            );

                            break;
                        }
                        Ordering::Less => {
                            let to_advance = self.read_offset - current_chunk.offset;

                            trace!("the reader has already read past the start of the current chunk, advancing by {} bytes", to_advance);
                            current_chunk.try_advance(to_advance);
                            if current_chunk.is_empty() {
                                PeekMut::pop(current_chunk);
                                continue;
                            }
                        }
                        Ordering::Equal => {}
                    }

                    let bytes_to_read_from_chunk = cmp::min(buf.len(), current_chunk.bytes.len());

                    let local_buf = buf;
                    let (to_fill, remaining) = local_buf.split_at_mut(bytes_to_read_from_chunk);
                    to_fill.copy_from_slice(&current_chunk.bytes[..bytes_to_read_from_chunk]);
                    buf = remaining;

                    current_chunk.advance(bytes_to_read_from_chunk);
                    if current_chunk.is_empty() {
                        PeekMut::pop(current_chunk);
                    }

                    read_bytes += bytes_to_read_from_chunk;
                    self.read_offset += bytes_to_read_from_chunk;
                }
            }
        }

        read_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::DataQueue;
    use bytes::Bytes;

    #[test]
    fn read_of_empty_returns_zero_bytes_read() {
        let mut data_queue = DataQueue::new();

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(read_bytes, 0);
    }

    #[test]
    fn read_of_first_chunk() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, false, Bytes::from_static(b"hello world"));

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b"hello world");
    }

    #[test]
    fn read_past_first_chunk() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, false, Bytes::from("hello"));
        data_queue.insert_chunk(5, false, Bytes::from(" world"));

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b"hello world");
    }

    #[test]
    fn read_partial_chunks() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, false, Bytes::from("hello"));
        data_queue.insert_chunk(5, false, Bytes::from(" world"));

        let mut buf = [0; 4];

        let read_bytes = data_queue.read(&mut buf);
        assert_eq!(&buf[..read_bytes], b"hell");

        let read_bytes = data_queue.read(&mut buf);
        assert_eq!(&buf[..read_bytes], b"o wo");

        let read_bytes = data_queue.read(&mut buf);
        assert_eq!(&buf[..read_bytes], b"rld");
    }

    #[test]
    fn read_with_chunks_with_gap() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, false, Bytes::from("hello"));
        data_queue.insert_chunk(7, false, Bytes::from("orld"));

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b"hello");
    }

    #[test]
    fn insert_chunk_out_of_order() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(5, false, Bytes::from(" world"));
        data_queue.insert_chunk(0, false, Bytes::from("hello"));

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b"hello world");
    }

    #[test]
    fn read_late_filled_gap() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, false, Bytes::from("hello"));
        data_queue.insert_chunk(7, false, Bytes::from("orld"));

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b"hello");

        data_queue.insert_chunk(5, false, Bytes::from(" w"));

        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b" world");
    }

    #[test]
    fn read_overlapping_chunks() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, false, Bytes::from("hello"));
        data_queue.insert_chunk(2, false, Bytes::from("llo world"));

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b"hello world");
    }

    #[test]
    fn is_finished_with_no_data_returns_true() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, true, Bytes::new());

        assert!(data_queue.is_finished());
    }

    #[test]
    fn is_finished_before_read_all_data_returns_false() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, true, Bytes::from("hello world"));

        assert_eq!(data_queue.is_finished(), false);
    }

    #[test]
    fn is_finished_after_read_all_data_returns_true() {
        let mut data_queue = DataQueue::new();

        data_queue.insert_chunk(0, true, Bytes::from("hello world"));

        let mut buf = [0; 1024];
        let read_bytes = data_queue.read(&mut buf);

        assert_eq!(&buf[..read_bytes], b"hello world");
        assert!(data_queue.is_finished());
    }

}

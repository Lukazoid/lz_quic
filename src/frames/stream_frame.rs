use errors::*;
use stream_id::StreamId;
use frames::stream_offset::StreamOffset;

#[derive(Debug, Clone)]
pub struct StreamFrame {
    pub finished: bool,
    pub offset: StreamOffset,
    pub stream_id: StreamId,
    pub data: Vec<u8>,
}


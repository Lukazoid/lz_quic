use frames::StreamOffset;
use protocol::StreamId;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StreamFrame {
    pub finished: bool,
    pub offset: StreamOffset,
    pub stream_id: StreamId,
    pub data: Vec<u8>,
}

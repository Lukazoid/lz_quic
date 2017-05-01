use errors::*;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};
use stream_id::{StreamId, StreamIdLength};
use frames::stream_offset::StreamOffset;

#[derive(Debug, Clone)]
pub struct StreamFrame {
    pub finished: bool,
    pub offset: StreamOffset,
    pub stream_id: StreamId,
    pub data: Vec<u8>,
}


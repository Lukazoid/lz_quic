use tokio_io::{AsyncRead, AsyncWrite};
use std::io::{Read, Write};
use stream_id::StreamId;

#[derive(Debug)]
pub struct DataStream {
    stream_id: StreamId,
}


// impl Read for DataStream {}

// impl AsyncRead for DataStream {}

// impl Write for DataStream {}

// impl AsyncWrite for DataStream {}


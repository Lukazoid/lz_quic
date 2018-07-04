use errors::*;
use futures::stream::Stream;
use futures::Poll;
use std::sync::Arc;
use {Connection, DataStream, Perspective};

#[derive(Debug)]
pub struct NewDataStreams<P: Perspective> {
    connection: Arc<Connection<P>>,
}

impl<P: Perspective> NewDataStreams<P> {
    pub(crate) fn new(connection: Arc<Connection<P>>) -> Self {
        Self {
            connection: connection,
        }
    }
}

impl<P: Perspective> Stream for NewDataStreams<P> {
    type Item = DataStream<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

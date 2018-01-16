use errors::*;
use {DataStream, Connection};
use futures::stream::Stream;
use futures::Poll;
use std::sync::Arc;

#[derive(Debug)]
pub struct NewDataStreams<P> {
    connection: Arc<Connection<P>>,
}

impl<P> NewDataStreams<P> {
    pub(crate) fn new(connection: Arc<Connection<P>>) -> Self {
        Self { connection: connection }
    }
}

impl<P> Stream for NewDataStreams<P> {
    type Item = DataStream<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

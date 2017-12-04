use errors::*;
use {DataStream, Session};
use futures::stream::Stream;
use futures::Poll;
use std::sync::Arc;

#[derive(Debug)]
pub struct NewDataStreams<P> {
    session: Arc<Session<P>>,
}

impl<P> NewDataStreams<P> {
    pub(crate) fn new(session: Arc<Session<P>>) -> Self {
        Self { session: session }
    }
}

impl<P> Stream for NewDataStreams<P> {
    type Item = DataStream<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

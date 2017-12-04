use errors::*;
use {DataStream, Session};
use futures::{Future, Poll};
use std::sync::Arc;

#[derive(Debug)]
pub struct NewDataStream<P> {
    session: Arc<Session<P>>,
}

impl<P> NewDataStream<P> {
    pub(crate) fn new(session: Arc<Session<P>>) -> Self {
        Self { session: session }
    }
}

impl<P> Future for NewDataStream<P> {
    type Item = DataStream<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unimplemented!()
    }
}

use errors::*;
use {DataStream, Connection};
use futures::{Future, Poll};
use std::sync::Arc;

#[derive(Debug)]
pub struct NewDataStream<P> {
    connection: Arc<Connection<P>>,
}

impl<P> NewDataStream<P> {
    pub(crate) fn new(connection: Arc<Connection<P>>) -> Self {
        Self { connection: connection }
    }
}

impl<P> Future for NewDataStream<P> {
    type Item = DataStream<P>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unimplemented!()
    }
}

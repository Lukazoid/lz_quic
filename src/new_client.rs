use Client;
use errors::*;
use futures::{Future, Poll};

pub type BoxedFuture = Box<Future<Item = Client, Error = Error> + Send>;

pub struct NewClient {
    inner: BoxedFuture,
}

impl NewClient {
    pub(crate) fn new(inner: BoxedFuture) -> Self {
        NewClient { inner: inner }
    }
}

impl Future for NewClient {
    type Item = Client;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}

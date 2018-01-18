use errors::*;
use {Connection, DataStream, Perspective};
use protocol::StreamId;
use std::sync::Arc;
use futures::Future;

pub trait SharedConnection<P> {
    fn handshake(self) -> Box<Future<Item = (), Error = Error> + Send>;
}

impl<P: Perspective + 'static> SharedConnection<P> for Arc<Connection<P>>
where
    P::TlsSession: 'static,
{
    fn handshake(self) -> Box<Future<Item = (), Error = Error> + Send> {
        let crypto_stream = DataStream::new(StreamId::from(0), self.clone());

        (*self).handshake(crypto_stream)
    }
}

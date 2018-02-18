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
        let (stream_id, stream_state) = self.crypto_stream();

        let crypto_stream = DataStream::new(stream_id, self.clone(), stream_state);

        (*self).handshake(crypto_stream)
    }
}

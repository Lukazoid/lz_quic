use errors::*;
use {Connection, DataStream, Perspective, StreamMapEntry};
use std::sync::Arc;
use futures::Future;
use futures::future;

pub trait SharedConnection<P> {
    fn handshake(self) -> Box<Future<Item = (), Error = Error> + Send>;
}

impl<P: Perspective + 'static> SharedConnection<P> for Arc<Connection<P>>
where
    P::TlsSession: 'static,
{
    fn handshake(self) -> Box<Future<Item = (), Error = Error> + Send> {
        let (stream_id, stream_map_entry) = self.crypto_stream();
        match stream_map_entry {
            StreamMapEntry::Dead => {
                Box::new(future::err(ErrorKind::CryptoStreamAlreadyClosed.into()))
            }
            StreamMapEntry::Live(stream_state) => {
                let crypto_stream = DataStream::new(stream_id, self.clone(), stream_state);

                (*self).handshake(crypto_stream)
            }
        }
    }
}

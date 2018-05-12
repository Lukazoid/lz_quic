use errors::*;
use futures::Future;
use futures::future;
use std::sync::Arc;
use {Connection, DataStream, Perspective, StreamMapEntry};

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

                Box::new((*self).handshake(crypto_stream))
            }
        }
    }
}

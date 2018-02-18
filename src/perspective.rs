use errors::*;
use {DataStream, StreamMap};
use rustls::Session;
use tokio_rustls::TlsStream;
use futures::Future;

pub trait Perspective {
    type TlsSession: Session;

    fn handshake(
        &self,
        crypto_stream: DataStream<Self>,
    ) -> Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>
    where
        Self: Sized;

    fn create_stream_map() -> StreamMap;
}

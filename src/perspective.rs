use errors::*;
use futures::Future;
use rustls::Session;
use tokio_rustls::TlsStream;
use {DataStream, StreamMap};

pub trait Perspective: Sized {
    type TlsSession: Session;
    type HandshakeFuture: Future<
        Item = TlsStream<DataStream<Self>, Self::TlsSession>,
        Error = Error,
    >
        + Send;

    fn handshake(&self, crypto_stream: DataStream<Self>) -> Self::HandshakeFuture;

    fn create_stream_map() -> StreamMap;
}

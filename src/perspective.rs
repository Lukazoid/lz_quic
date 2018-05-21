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

    fn tls_exporter_send_label() -> &'static str;

    fn tls_exporter_receive_label() -> &'static str;

    fn create_stream_map() -> StreamMap;
}

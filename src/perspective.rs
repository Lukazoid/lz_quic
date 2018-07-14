use errors::*;
use futures::{Future, Poll};
use packets::IncomingPacket;
use protocol::{ConnectionId, MessageParameters, Readable, Role, RoleSpecificTransportParameters};
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
    type IncomingTransportMessageParameters: MessageParameters;
    type RoleSpecificTransportParameters: RoleSpecificTransportParameters;

    fn handshake(&self, crypto_stream: DataStream<Self>) -> Self::HandshakeFuture;

    fn client_connection_id(
        local_connection_id: ConnectionId,
        remote_connection_id: ConnectionId,
    ) -> ConnectionId;

    fn handshake_send_label() -> &'static str;

    fn handshake_receive_label() -> &'static str;

    fn update_secret_send_label() -> &'static str;

    fn update_secret_receive_label() -> &'static str;

    fn tls_exporter_send_label() -> &'static str;

    fn tls_exporter_receive_label() -> &'static str;

    fn create_stream_map() -> StreamMap;

    fn poll_incoming_packet(&self, connection_id: ConnectionId) -> Poll<IncomingPacket, Error>;

    fn role() -> Role;

    fn max_incoming_data_per_stream(&self) -> u32;

    fn max_incoming_data(&self) -> u32;
}

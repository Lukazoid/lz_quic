use protocol::StreamType;
use std::sync::Arc;
use {Connection, DataStream, NewDataStreams, ServerPerspective};

/// A client which has connected to this `Server`.
#[derive(Debug)]
pub struct RemoteClient {
    connection: Arc<Connection<ServerPerspective>>,
}

impl RemoteClient {
    fn open_stream(&self, stream_type: StreamType) -> DataStream<ServerPerspective> {
        let (stream_id, stream_state) = self.connection.new_stream(stream_type);

        DataStream::new(stream_id, self.connection.clone(), stream_state)
    }

    fn open_unidirectional_stream(&self) -> DataStream<ServerPerspective> {
        self.open_stream(StreamType::Unidirectional)
    }

    fn open_bidirectional_stream(&self) -> DataStream<ServerPerspective> {
        self.open_stream(StreamType::Bidirectional)
    }

    pub fn incoming_streams(&self) -> NewDataStreams<ServerPerspective> {
        NewDataStreams::new(self.connection.clone())
    }
}

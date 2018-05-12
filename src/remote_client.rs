use {Connection, DataStream, NewDataStreams, ServerPerspective};
use std::sync::Arc;

/// A client which has connected to this `Server`.
#[derive(Debug)]
pub struct RemoteClient {
    connection: Arc<Connection<ServerPerspective>>,
}

impl RemoteClient {
    pub fn open_stream(&self) -> DataStream<ServerPerspective> {
        let (stream_id, stream_state) = self.connection.new_stream();

        DataStream::new(stream_id, self.connection.clone(), stream_state)
    }

    pub fn incoming_streams(&self) -> NewDataStreams<ServerPerspective> {
        NewDataStreams::new(self.connection.clone())
    }
}

use {NewDataStream, NewDataStreams, Session, ServerPerspective};
use std::sync::Arc;

/// A client which has connected to this `Server`.
#[derive(Debug)]
pub struct RemoteClient {
    session: Arc<Session<ServerPerspective>>,
}

impl RemoteClient {
    pub fn open_stream(&self) -> NewDataStream<ServerPerspective> {
        unimplemented!()
    }

    pub fn incoming_streams(&self) -> NewDataStreams<ServerPerspective> {
        unimplemented!()
    }
}
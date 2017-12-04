use errors::*;
use RemoteClient;
use futures::Poll;
use futures::stream::Stream;

#[derive(Debug)]
pub struct NewRemoteClients {}

impl Stream for NewRemoteClients {
    type Item = RemoteClient;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

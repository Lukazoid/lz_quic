use errors::*;
use futures::stream::Stream;
use futures::Poll;
use RemoteClient;

#[derive(Debug)]
pub struct NewRemoteClients {}

impl Stream for NewRemoteClients {
    type Item = RemoteClient;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

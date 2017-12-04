use tokio_io::codec::{Decoder, Encoder};
use std::io::{Error as IoError, Cursor};
use handshake::HandshakeMessage;
use protocol::{Readable, Writable};
use bytes::BytesMut;

/// A codec for reading/writing `HandshakeMessage` values to/from a data stream.
#[derive(Debug, Default, Clone)]
pub struct HandshakeCodec;

impl Decoder for HandshakeCodec {
    type Item = HandshakeMessage;
    type Error = IoError;

    fn decode(
        &mut self, 
        src: &mut BytesMut
    ) -> Result<Option<Self::Item>, Self::Error>{
        if src.is_empty() {
            Ok(None)
        } else {
            let handshake_message;
            let bytes_read;
            {
                let mut cursor = Cursor::new(src.as_ref());
                handshake_message = HandshakeMessage::read(&mut cursor).ok();
                bytes_read = cursor.position() as usize;
            }

            if handshake_message.is_some() {
                // If we read a handshake message, move the buffer forward
                src.split_to(bytes_read);
            }

            Ok(handshake_message)
        }
    }
}

impl Encoder for HandshakeCodec {
    type Item = HandshakeMessage;
    type Error = IoError;

    fn encode(
        &mut self, 
        item: Self::Item, 
        dst: &mut BytesMut
    ) -> Result<(), Self::Error> {
        let mut vec = Vec::new();
        item.write_to_vec(&mut vec);

        dst.extend_from_slice(vec.as_slice());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handshake::{HandshakeMessage, RejectionMessage};
    use std::io::Cursor;
    use tokio_io::{AsyncRead, AsyncWrite};
    use futures::Future;
    use futures::sink::Sink;
    use futures::stream::Stream;

    #[test]
    fn decode_decodes_previously_encoded() {
        let handshake_message = HandshakeMessage::Rejection(RejectionMessage {
            server_configuration: None,
            source_address_token: None,
            server_nonce: None,
            seconds_to_live: 3600u64,
            compressed_certificate_chain: None,
            server_proof: None,
        });

        let mut cursor = Cursor::new(Vec::new());

        let mut framed = cursor.framed(HandshakeCodec::default());

        // Write to the stream
        let read = framed.send(handshake_message.clone())
            .and_then(|mut framed| {
                framed.get_mut().set_position(0);

                // Read from the stream
                framed.into_future().map(|x|x.0)
                    .map_err(|x|x.0)
            })
            .wait()
            .unwrap()
            .unwrap();
        
        assert_eq!(read, handshake_message);

    }
}
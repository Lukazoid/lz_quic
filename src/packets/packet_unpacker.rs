use std::sync::{Arc, RwLock};
use handshake::CryptoStage;

#[derive(Debug)]
pub struct PacketUnpacker{    
    crypto_stage: Arc<RwLock<CryptoStage>>,
}

impl PacketUnpacker {

}
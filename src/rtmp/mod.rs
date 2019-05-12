// mod.
mod handshake;
mod session;


// use.
use bytes::Bytes;
use bytes::BytesMut;
use std::sync::mpsc::Sender;
use session::Session;
use handshake::Handshakes;
use rml_rtmp::sessions::StreamMetadata;


#[derive(Clone)]
pub struct Crated {
    pub name: String,
    pub key: String,
    pub meta: StreamMetadata
}


#[derive(Clone)]
pub struct Metadata {
    pub name: String,
    pub key: String,
    pub data: Bytes
}


#[derive(Clone)]
pub enum Message {
    Raw(Bytes),
    Metadata(Metadata),
    Crated(Crated)
}


/// # RTMP Control.
pub struct Rtmp {
    pub sender: Sender<Message>,
    pub handshake: Handshakes,
    pub session: Session
}


impl Rtmp {

    /// # Create RTMP.
    pub fn new (address: String, sender: Sender<Message>) -> Self {
        let session = Session::new(address, Sender::clone(&sender));
        let handshake = Handshakes::new(Sender::clone(&sender));
        Rtmp { handshake, session, sender }
    }

    /// # Decoder Bytes.
    /// processing RTMP data.
    pub fn decoder (&mut self, bytes: BytesMut) { 
        let mut bytes_copy = bytes.clone().to_vec();

        // handshake.
        if self.handshake.completed == false {
            self.handshake.process(&mut bytes_copy);
        }

        // message.
        if self.handshake.completed == true && bytes_copy.len() > 0 {
            self.session.process(bytes_copy.clone());
        }
    }
}
use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct Name {
    #[deku(update = "ClientMessage::Name")]
    pub msg_type: ClientMessage,

    pub username: [u8; 12],
}

impl Name {
    pub fn pene(&self) -> [u8; 13] {
        let mut buf = [0u8; 13];
        buf[0] = (ClientMessage::Name as u8) & 0x0F;
        buf[1..13].copy_from_slice(&self.username);
        buf
    }
}

use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct Room {
    #[deku(update = "ClientMessage::JoinRoom")]
    pub msg_type: ClientMessage,

    pub room: u8,
}

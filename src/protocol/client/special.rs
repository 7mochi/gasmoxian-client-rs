use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Special {
    #[deku(update = "ClientMessage::Special")]
    pub msg_type: ClientMessage,

    pub gamemodes: [bool; 18],
}

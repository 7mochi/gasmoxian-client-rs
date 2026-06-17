use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct Password {
    #[deku(update = "ClientMessage::Password")]
    pub msg_type: ClientMessage,

    pub sequence: [u8; 8],
}

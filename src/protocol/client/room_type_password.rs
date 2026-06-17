use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct RoomTypePassword {
    #[deku(update = "ClientMessage::RoomType")]
    pub msg_type: ClientMessage,

    pub room_type: u8,
    pub r_type_locked: u8,
    pub sequence: [u8; 8],
}

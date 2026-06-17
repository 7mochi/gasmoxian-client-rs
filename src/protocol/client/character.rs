use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct Character {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub character_id: u8,

    pub locked_in: bool,
}

impl Character {
    pub fn new(character_id: u8, locked_in: bool) -> Self {
        Self {
            _msg_type: ClientMessage::Character as u8,
            character_id,
            locked_in,
        }
    }
}

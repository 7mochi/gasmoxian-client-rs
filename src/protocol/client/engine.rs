use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct Engine {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _pad0: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub engine_type: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub locked_in: bool,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    _pad1: u8,

    _end: u8,
}

impl Engine {
    pub fn new(engine_type: u8, locked_in: bool) -> Self {
        Self {
            _msg_type: ClientMessage::Engine as u8,
            engine_type,
            locked_in,
            ..Default::default()
        }
    }
}

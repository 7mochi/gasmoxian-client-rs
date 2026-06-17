use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
pub struct Weapon {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub juiced: bool,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    _pad0: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub flags: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub weapon: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _end: u8,
}

impl Weapon {
    pub fn new(juiced: bool, flags: u8, weapon: u8) -> Self {
        Self {
            _msg_type: ClientMessage::Weapon as u8,
            juiced,
            flags,
            weapon,
            ..Default::default()
        }
    }
}

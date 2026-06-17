use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct WarpClock {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    _pad: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub warp_clock: u8,
}

impl WarpClock {
    pub fn new(warp_clock: u8) -> Self {
        Self {
            _msg_type: ClientMessage::Warpclock as u8,
            warp_clock,
            ..Default::default()
        }
    }
}

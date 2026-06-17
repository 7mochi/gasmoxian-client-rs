use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct FinishTimer {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _pad: u8,

    #[deku(bits = "6", ctx = "deku::ctx::Order::Lsb0")]
    pub finish_timer: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    _end: u8,
}

impl FinishTimer {
    pub fn new(finish_timer: u8) -> Self {
        Self {
            _msg_type: ClientMessage::FinishTimer as u8,
            finish_timer,
            ..Default::default()
        }
    }
}

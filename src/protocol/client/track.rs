use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct Track {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _pad0: u8,

    #[deku(bits = "5", ctx = "deku::ctx::Order::Lsb0")]
    pub track_id: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    _pad1: u8,

    pub lap_id: u8,
}

impl Track {
    pub fn new(track_id: u8, lap_id: u8) -> Self {
        Self {
            _msg_type: ClientMessage::Track as u8,
            track_id,
            lap_id,
            ..Default::default()
        }
    }
}

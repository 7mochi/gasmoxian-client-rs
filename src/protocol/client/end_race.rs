use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct EndRace {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(pad_bits_after = "4")]
    _pad0: (),

    #[deku(pad_bytes_after = "3")]
    _pad1: (),

    #[deku(endian = "little")]
    pub course_time: i32,

    #[deku(endian = "little")]
    pub lap_time: i32,
}

impl EndRace {
    pub fn new(course_time: i32, lap_time: i32) -> Self {
        Self {
            _msg_type: ClientMessage::EndRace as u8,
            course_time,
            lap_time,
            ..Default::default()
        }
    }
}

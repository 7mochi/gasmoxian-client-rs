use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
pub struct Kart {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub wumpa: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub reserves: bool,

    pub kart_rotation1: u8,
    pub kart_rotation2: u8,
    pub button_hold: u8,

    #[deku(endian = "little")]
    pub position_x: i16,

    #[deku(endian = "little")]
    pub position_y: i16,

    #[deku(endian = "little")]
    pub position_z: i16,
}

impl Kart {
    pub fn new(
        wumpa: u8,
        reserves: bool,
        kart_rotation1: u8,
        kart_rotation2: u8,
        button_hold: u8,
        position_x: i16,
        position_y: i16,
        position_z: i16,
    ) -> Self {
        Self {
            _msg_type: ClientMessage::RaceData as u8,
            wumpa: wumpa & 0x07,
            reserves,
            kart_rotation1,
            kart_rotation2,
            button_hold,
            position_x,
            position_y,
            position_z,
        }
    }
}

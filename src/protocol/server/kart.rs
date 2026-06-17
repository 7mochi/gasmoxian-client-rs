use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Kart {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub wumpa: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub reserves: bool,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(bits = "5", ctx = "deku::ctx::Order::Lsb0")]
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

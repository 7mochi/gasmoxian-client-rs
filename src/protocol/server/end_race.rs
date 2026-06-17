use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct EndRace {
    #[deku(pad_bytes_before = "1")]
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(pad_bits_after = "4")]
    _pad_bits: (),

    #[deku(pad_bytes_before = "2")]
    #[deku(endian = "little")]
    pub course_time: i32,

    #[deku(endian = "little")]
    pub lap_time: i32,
}

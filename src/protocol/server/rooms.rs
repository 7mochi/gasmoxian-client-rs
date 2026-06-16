use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Rooms {
    #[deku(pad_bytes_before = "1")]
    pub room_count: u8,

    #[deku(endian = "little")]
    pub version: u16,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_count: [u8; 16],
}

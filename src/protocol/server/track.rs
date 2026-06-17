use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Track {
    #[deku(pad_bytes_before = "1")]
    #[deku(bits = "5", ctx = "deku::ctx::Order::Lsb0")]
    pub track_id: u8,

    #[deku(pad_bits_before = "3")]
    pub lap_id: u8,
}

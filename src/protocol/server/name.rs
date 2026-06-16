use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Name {
    #[deku(pad_bytes_before = "1")]
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_count: u8,

    pub username: [u8; 12],
}

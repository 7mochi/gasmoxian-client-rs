use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct ClientStatus {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_count: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub _pad: u8,
}

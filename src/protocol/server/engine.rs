use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Engine {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(pad_bits_after = "1")]
    _pad0: (),

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub engine_type: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub locked_in: bool,

    #[deku(pad_bits_after = "3")]
    _pad1: (),
}

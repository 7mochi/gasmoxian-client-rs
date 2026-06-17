use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Weapon {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub juiced: bool,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub weapon: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub flags: u8,

    #[deku(pad_bits_after = "2")]
    _pad: (),
}

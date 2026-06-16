use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct ClientStatus {
    #[deku(bits = "4")]
    pub msg_type: u8,
    #[deku(bits = "4")]
    pub client_id: u8,
    #[deku(bits = "4")]
    pub client_count: u8,
    #[deku(bits = "4")]
    pub _pad: u8,
}

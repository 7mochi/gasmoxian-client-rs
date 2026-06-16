use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct RoomType {
    #[deku(pad_bytes_before = "1")]
    pub room_type: u8,
    pub r_type_locked: u8,
}

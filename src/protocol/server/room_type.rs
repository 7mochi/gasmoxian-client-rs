/// Confirms the room type assignment.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// |               0               |                  1                  |                   2                   |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// |   _msg_type   |     _pad      |              room_type              |             r_type_locked             |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
///
///  Field           Bits   Offset     Description
///  _msg_type       4      byte 0:0   ServerMessage::RoomType
///  _pad            4      byte 0:4   Unused
///  room_type       8      byte 1:0   0=normal, 1=tournament
///  r_type_locked   8      byte 2:0   1=locked by host
use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(endian = "little")]
pub struct RoomType {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(pad_bits_after = "4")]
    _pad: (),

    pub room_type: u8,

    pub r_type_locked: u8,
}

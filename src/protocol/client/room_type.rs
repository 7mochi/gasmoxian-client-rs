/// Sent by the host to change room type.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// |               0               |                  1                  |                   2                   |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// |           _msg_type           |              room_type              |             r_type_locked             |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
///
///  Field           Bits   Offset     Description 
///  _msg_type       8      byte 0:0   ClientMessage::RoomType
///  room_type       8      byte 1:0   0=normal, 1=tournament
///  r_type_locked   8      byte 2:0   1=locked
use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct RoomType {
    _msg_type: u8,

    pub room_type: u8,
    pub r_type_locked: u8,
}

impl RoomType {
    pub fn new(room_type: u8, r_type_locked: u8) -> Self {
        Self {
            _msg_type: ClientMessage::RoomType as u8,
            room_type,
            r_type_locked,
        }
    }
}


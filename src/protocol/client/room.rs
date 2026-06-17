/// Sent when joining a room from the lobby.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |               0               |                  1                  |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |           _msg_type           |                room                 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
///
///  Field      Bits   Offset     Description
///  _msg_type  8      byte 0:0   ClientMessage::JoinRoom
///  room       8      byte 1:0   Room index 0-15
use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct Room {
    _msg_type: u8,

    pub room: u8,
}

impl Room {
    /// Creates a join room message for the given room index (0-15).
    /// Passing `0xFF` triggers a room list refresh.
    pub fn new(room: u8) -> Self {
        Self {
            _msg_type: ClientMessage::JoinRoom as u8,
            room,
        }
    }
}

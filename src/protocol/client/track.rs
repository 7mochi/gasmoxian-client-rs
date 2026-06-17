/// Sent by the host when selecting a track and lap count.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// |               0               |                  1                  |                   2                   |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
/// |   _msg_type   |     _pad0     |       track_id       |    _pad1     |                lap_id                 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
///
///  Field       Bits   Offset     Description
///  _msg_type   4      byte 0:0   ClientMessage::Track
///  _pad0       4      byte 0:4   Unused
///  track_id    5      byte 1:0   Track ID 0-24
///  _pad1       3      byte 1:5   Unused
///  lap_id      8      byte 2:0   Laps: 0=1, 1=3, 2=5, 3=7, 4=9, 5=11, 6=13, 7=15 (2*lapID+1)
use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct Track {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _pad0: u8,

    #[deku(bits = "5", ctx = "deku::ctx::Order::Lsb0")]
    pub track_id: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    _pad1: u8,

    pub lap_id: u8,
}

impl Track {
    /// Creates a track selection message.
    ///
    /// `track_id` is the track index. `lap_id` encodes the lap count:
    /// 0=1, 1=3, 2=5, 3=7, 4+=special lap values from a lookup table.
    pub fn new(track_id: u8, lap_id: u8) -> Self {
        Self {
            _msg_type: ClientMessage::Track as u8,
            track_id,
            lap_id,
            ..Default::default()
        }
    }
}

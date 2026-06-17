/// Sent when the host selects a track and lap count for the next race.
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
///  _msg_type   4      byte 0:0   ServerMessage::Track
///  _pad0       4      byte 0:4   Unused
///  track_id    5      byte 1:0   Track ID (TODO: put values here)
///  _pad1       3      byte 1:5   Unused
///  lap_id      8      byte 2:0   Number of laps (TODO: confirm values 0 = 1 lap, 6 = 7 laps, etc.)
use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Track {
    #[deku(pad_bytes_before = "1")]
    #[deku(bits = "5", ctx = "deku::ctx::Order::Lsb0")]
    pub track_id: u8,

    #[deku(pad_bits_before = "3")]
    pub lap_id: u8,
}

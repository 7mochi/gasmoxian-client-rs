/// Finish countdown timer sync.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |               0               |                  1                  |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |   _msg_type   |     _pad      |       finish_timer        |  _pad2  |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
///
///  Field          Bits   Offset     Description 
///  _msg_type      4      byte 0:0   ServerMessage::FinishTimer
///  _pad           4      byte 0:4   Unused
///  finish_timer   6      byte 1:0   Countdown (0-63)
///  _pad2          2      byte 1:6   Unused
use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct FinishTimer {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(pad_bits_after = "4")]
    _pad: (),

    #[deku(bits = "6", ctx = "deku::ctx::Order::Lsb0")]
    pub finish_timer: u8,

    #[deku(pad_bits_after = "2")]
    _pad2: (),
}

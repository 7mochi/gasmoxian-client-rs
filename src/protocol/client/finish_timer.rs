/// Sent to sync the finish countdown timer during a race.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |               0               |                  1                  |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |   _msg_type   |     _pad0     |       finish_timer        |  _pad1  |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
///
///  Field          Bits   Offset     Description
///  _msg_type      4      byte 0:0   ClientMessage::FinishTimer
///  _pad0          4      byte 0:4   Unused
///  finish_timer   6      byte 1:0   Finish timer (0-63)
///  _pad1          2      byte 1:6   Unused
use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct FinishTimer {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _pad0: u8,

    #[deku(bits = "6", ctx = "deku::ctx::Order::Lsb0")]
    pub finish_timer: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    _pad1: u8,
}

impl FinishTimer {
    /// Creates a finish countdown sync message.
    /// `finish_timer` is the remaining countdown value (0-63).
    pub fn new(finish_timer: u8) -> Self {
        Self {
            _msg_type: ClientMessage::FinishTimer as u8,
            finish_timer,
            ..Default::default()
        }
    }
}

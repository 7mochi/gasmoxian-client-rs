/// Sent to broadcast the player's warp clock state.
///
/// +---+---+---+---+---+---+------+-----+
/// |                 0                  |
/// +---+---+---+---+---+---+------+-----+
/// | 0 | 1 | 2 | 3 | 4 | 5 |  6   |  7  |
/// +---+---+---+---+---+---+------+-----+
/// |   _msg_type   | _pad  | warp_clock |
/// +---+---+---+---+---+---+------+-----+
///
///  Field        Bits   Offset     Description
///  _msg_type    4      byte 0:0   ClientMessage::Warpclock
///  _pad         2      byte 0:4   Unused
///  warp_clock   2      byte 0:6   0=inactive, 1=active (warp orb/clock event)
use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct WarpClock {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    _pad: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub warp_clock: u8,
}

impl WarpClock {
    /// Creates a warp clock state notification.
    /// `warp_clock`: 0=inactive, 1=active (warp orb/clock event).
    pub fn new(warp_clock: u8) -> Self {
        Self {
            _msg_type: ClientMessage::Warpclock as u8,
            warp_clock,
            ..Default::default()
        }
    }
}

/// Warp clock state during a race.
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
///  _msg_type    4      byte 0:0   ServerMessage::Warpclock
///  _pad         2      byte 0:4   Unused
///  warp_clock   2      byte 0:6   Warp orb/clock event (0=inactive, 1=active)
use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct WarpClock {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(pad_bits_after = "2")]
    _pad: (),

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub warp_clock: u8,
}

/// Sent when the player picks up or uses a weapon item.
///
/// +---+---+---+---+-----+-------+---+---+---+---+----+----+----+----+----+----+
/// |                  0                  |                  1                  |
/// +---+---+---+---+-----+-------+---+---+---+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 |  4  |   5   | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+-----+-------+---+---+---+---+----+----+----+----+----+----+
/// |   _msg_type   | jcd | _pad0 | flags |     weapon      |       _end        |
/// +---+---+---+---+-----+-------+---+---+---+---+----+----+----+----+----+----+
///
///  Field       Bits   Offset     Description 
///  _msg_type   4      byte 0:0   ClientMessage::Weapon
///  jcd         1      byte 0:4   1=juiced (powered up)
///  _pad0       1      byte 0:5   Unused
///  flags       2      byte 0:6   Aim flags
///  weapon      4      byte 1:0   Weapon ID (0=TurboBoost, 1=BowlingBoom, 2=TrackingMissile, etc.)
///  _end        4      byte 1:4   Unused
use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Default, DekuRead, DekuWrite)]
pub struct Weapon {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub juiced: bool,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    _pad0: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub flags: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub weapon: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _end: u8,
}

impl Weapon {
    pub fn new(juiced: bool, flags: u8, weapon: u8) -> Self {
        Self {
            _msg_type: ClientMessage::Weapon as u8,
            juiced,
            flags,
            weapon,
            ..Default::default()
        }
    }
}

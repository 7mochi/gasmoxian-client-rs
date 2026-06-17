/// Sent when the player selects or locks in an engine type.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
/// |               0               |                  1                   |                   2                   |
/// +---+---+---+---+---+---+---+---+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12  | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
/// |   _msg_type   |     _pad0     |   engine_type   | lck |                        _pad1                         |
/// +---+---+---+---+---+---+---+---+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
///
///  Field         Bits   Offset     Description 
///  _msg_type     4      byte 0:0   ClientMessage::Engine
///  _pad0         4      byte 0:4   Unused
///  engine_type   4      byte 1:0   0=BALANCED, 1=ACCEL, 2=SPEED, 3=TURN
///  lck           1      byte 1:4   1=locked in
///  _pad1         11     byte 1:5   Unused
use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct Engine {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _pad0: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub engine_type: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub locked_in: bool,

    #[deku(bits = "11", ctx = "deku::ctx::Order::Lsb0")]
    _pad1: u16,
}

impl Engine {
    pub fn new(engine_type: u8, locked_in: bool) -> Self {
        Self {
            _msg_type: ClientMessage::Engine as u8,
            engine_type,
            locked_in,
            ..Default::default()
        }
    }
}

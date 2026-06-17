/// Sent when a player selects an engine.
///
/// +---+---+---+---+---+---+---+-------+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
/// |                 0                 |                  1                   |                   2                   |
/// +---+---+---+---+---+---+---+-------+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 |   7   | 8 | 9 | 10 | 11 | 12  | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
/// +---+---+---+---+---+---+---+-------+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
/// |   _msg_type   | client_id | _pad0 |   engine_type   | Lck |                        _pad1                         |
/// +---+---+---+---+---+---+---+-------+---+---+----+----+-----+----+----+----+----+----+----+----+----+----+----+----+
///
///  Field         Bits   Offset     Description
///  _msg_type     4      byte 0:0   ServerMessage::Engine
///  client_id     3      byte 0:4   Driver slot 0-7
///  _pad0         1      byte 0:7   Unused
///  engine_type   4      byte 1:0   0=Balanced, 1=Accel, 2=Speed, 3=Turn
///  Lck           1      byte 1:4   1=locked in
///  _pad1         11     byte 1:5   Unused
use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Engine {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(pad_bits_after = "1")]
    _pad0: (),

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub engine_type: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub locked_in: bool,

    #[deku(pad_bits_after = "11")]
    _pad1: (),
}

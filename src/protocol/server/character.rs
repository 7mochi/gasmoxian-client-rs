/// Sent when a player selects a character.
///
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
/// |                0                |                  1                  |
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 |  7  | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
/// |   _msg_type   | client_id | Lck |  character_id   |       _pad        |
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
///
///  Field          Bits   Offset     Description
///  _msg_type      4      byte 0:0   ServerMessage::Character
///  client_id      3      byte 0:4   Driver slot (0-7)
///  lck            1      byte 0:7   1 = locked in, 0 = still cycling
///  character_id   4      byte 1:0   Character ID (0=Crash, 1=…)
///  _pad           4      byte 1:4   Unused
use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Character {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub locked_in: bool,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub character_id: u8,

    #[deku(pad_bits_after = "4")]
    _pad: (),
}

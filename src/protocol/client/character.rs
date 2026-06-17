/// Sent when the player selects or locks in a character in the lobby.
///
/// +---+---+---+---+---+---+---+---+-----+---+----+----+----+----+----+----+
/// |               0               |                   1                   |
/// +---+---+---+---+---+---+---+---+-----+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |  8  | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+---+---+---+---+-----+---+----+----+----+----+----+----+
/// |   _msg_type   | character_id  | lck |              _pad               |
/// +---+---+---+---+---+---+---+---+-----+---+----+----+----+----+----+----+
///
///  Field          Bits   Offset     Description
///  _msg_type      4      byte 0:0   ClientMessage::Character
///  character_id   4      byte 0:4   Character ID 0-15 (0=Crash, 1=Cortex, 2=Tiny, etc.)
///  lck            1      byte 1:0   1=locked in, 0=cycling
///  _pad           7      byte 1:1   Unused
use deku::prelude::*;

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, Default, DekuRead, DekuWrite)]
pub struct Character {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub character_id: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub locked_in: bool,

    #[deku(pad_bits_after = "7")]
    _pad: (),
}

impl Character {
    /// Creates a character selection message.
    ///
    /// `character_id` is the selected character (0=Crash, 1=Cortex,
    /// 2=Tiny, etc.). `locked_in` signals the server to finalise the
    /// choice and move to engine selection.
    pub fn new(character_id: u8, locked_in: bool) -> Self {
        Self {
            _msg_type: ClientMessage::Character as u8,
            character_id,
            locked_in,
            _pad: (),
        }
    }
}

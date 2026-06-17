/// Server → Client: Weapon pickup notification (2 bytes)
///
/// Sent when a player picks up or uses a weapon item.
///
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
/// |                0                |                  1                  |
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 |  7  | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
/// |   _msg_type   | client_id | jcd |     weapon      |  flags  |  _pad   |
/// +---+---+---+---+---+---+---+-----+---+---+----+----+----+----+----+----+
///
///  Field       Bits   Offset     Description 
///  _msg_type   4      byte 0:0   ServerMessage::Weapon
///  client_id   3      byte 0:4   Driver slot (0-7)
///  jcd         1      byte 0:7   1 = juiced (powered-up) weapon
///  weapon      4      byte 1:0   Weapon ID (TODO: confirm values 0=Bomb, 1=…..)
///  flags       2      byte 1:4   Weapon flags (TODO: confirm values aim direction etc.)
///  _pad        2      byte 1:6   Unused
use deku::{DekuRead, DekuWrite};

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Weapon {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "3", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(bits = "1", ctx = "deku::ctx::Order::Lsb0")]
    pub juiced: bool,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub weapon: u8,

    #[deku(bits = "2", ctx = "deku::ctx::Order::Lsb0")]
    pub flags: u8,

    #[deku(pad_bits_after = "2")]
    _pad: (),
}

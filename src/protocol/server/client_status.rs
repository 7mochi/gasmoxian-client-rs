/// Sent when a client joins or leaves the room.
///
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |               0               |                  1                  |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
/// |   _msg_type   |   client_id   |  client_count   |       _pad        |
/// +---+---+---+---+---+---+---+---+---+---+----+----+----+----+----+----+
///
///  Field          Bits   Offset     Description
///  _msg_type      4      byte 0:0   ServerMessage::NewClient
///  client_id      4      byte 0:4   Driver slot 0-7
///  client_count   4      byte 1:0   Total clients in room
///  _pad           4      byte 1:4   Unused
use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct ClientStatus {
    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    _msg_type: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_id: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub client_count: u8,

    #[deku(bits = "4", ctx = "deku::ctx::Order::Lsb0")]
    pub _pad: u8,
}

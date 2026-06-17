/// Signal: client is ready to start the race.
///
/// +---+---+---+---+---+---+---+---+
/// |               0               |
/// +---+---+---+---+---+---+---+---+
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
/// +---+---+---+---+---+---+---+---+
/// |           _msg_type           |
/// +---+---+---+---+---+---+---+---+
///
///  Field      Bits   Offset     Description
///  _msg_type  8      byte 0:0   ClientMessage::StartRace
use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct StartRace {
    _msg_type: u8,
}

impl Default for StartRace {
    fn default() -> Self {
        Self::new()
    }
}

impl StartRace {
    /// Creates an empty signal indicating the client is ready to race.
    pub fn new() -> Self {
        Self {
            _msg_type: ClientMessage::StartRace as u8,
        }
    }
}

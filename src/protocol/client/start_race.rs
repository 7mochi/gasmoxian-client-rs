use deku::{DekuRead, DekuWrite};

use crate::protocol::ClientMessage;

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct StartRace {
    #[deku(update = "ClientMessage::StartRace")]
    pub msg_type: ClientMessage,
}

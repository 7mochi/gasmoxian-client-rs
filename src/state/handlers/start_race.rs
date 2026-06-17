use crate::{effect::Effect, protocol::ClientState};

pub fn handle() -> Vec<Effect> {
    vec![Effect::SetState(ClientState::GameStartRace)]
}

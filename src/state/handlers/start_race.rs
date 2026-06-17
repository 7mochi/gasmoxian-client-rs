use crate::{effect::Effect, protocol::ClientState};

/// Handles `ServerMessage::StartRace`. Transitions to `GameStartRace`
/// to begin the active racing state.
pub fn handle() -> Vec<Effect> {
    vec![Effect::SetState(ClientState::GameStartRace)]
}

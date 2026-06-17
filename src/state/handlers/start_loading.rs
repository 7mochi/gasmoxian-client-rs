use crate::{effect::Effect, protocol::ClientState};

/// Handles `ServerMessage::StartLoading`. Transitions to
/// `LobbyStartLoading` so the client prepares for the race.
pub fn handle() -> Vec<Effect> {
    vec![Effect::SetState(ClientState::LobbyStartLoading)]
}

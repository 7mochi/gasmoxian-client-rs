use crate::{effect::Effect, protocol::ClientState, state::GameState};

pub fn handle(state: &mut GameState) -> Vec<Effect> {
    state.race.flags.lock_engine_and_character = false;
    state.race.flags.password_sent = false;

    vec![
        Effect::LogErr("Wrong password, returning to room list.".into()),
        Effect::DisconnectNow,
        Effect::SetRoomType(0),
        Effect::SetRoomTypeLocked(0),
        Effect::SetAutoRetryRoomIndex(-1),
        Effect::SetState(ClientState::LaunchPickRoom),
        Effect::SetPasswordEntered([0u8; 8]),
        Effect::SetRoomPasswordSequence([0u8; 8]),
    ]
}

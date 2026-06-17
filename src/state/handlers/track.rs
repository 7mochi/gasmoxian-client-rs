use crate::{
    effect::Effect,
    protocol::{ClientState, server::Track},
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

/// Handles `ServerMessage::Track`. Converts `lap_id` to the actual
/// lap count using the same lookup as the C server, writes it to PS1
/// memory, and transitions to `LobbySpecialPick`.
pub fn handle(_ctr: &OnlineCtrSnapshot, state: &mut GameState, message: Track) -> Vec<Effect> {
    let num_laps = if message.lap_id >= 4 && message.lap_id <= 15 {
        let lap_values = [10, 15, 20, 25, 30, 35, 40, 50, 69, 80, 90, 127];
        lap_values[(message.lap_id - 4) as usize]
    } else {
        (message.lap_id * 2) + 1
    };

    state.race.extra_laps = if message.lap_id >= 8 { 1 } else { 0 };

    vec![
        Effect::LogDebug(format!(
            "Setting track to {}, with {} laps",
            message.track_id, num_laps
        )),
        Effect::WriteU8(0x80096b20 + 0x1d33, num_laps),
        Effect::SetLevelId(message.track_id),
        Effect::SetState(ClientState::LobbySpecialPick),
    ]
}

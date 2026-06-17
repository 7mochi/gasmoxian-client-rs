use crate::{
    effect::Effect,
    protocol::{ClientState, server::RoomType},
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

pub fn handle(ctr: &OnlineCtrSnapshot, state: &mut GameState, message: RoomType) -> Vec<Effect> {
    let mut effects = vec![
        Effect::SetRoomType(message.room_type),
        Effect::LogDebug(format!("Room type set to {}", message.room_type)),
    ];

    if message.room_type == 2
        && ctr.room_type_locked == 0
        && ctr.current_state == ClientState::LaunchPickRoom as i32
    {
        effects.push(Effect::LogDebug(
            "Room type is password protected, waiting for password input.".to_string(),
        ));

        state.race.flags.password_sent = false;

        effects.push(Effect::SetState(ClientState::LaunchEnterPassword));
    }

    effects
}

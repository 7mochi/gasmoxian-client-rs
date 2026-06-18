use crate::{
    effect::Effect,
    protocol::{ClientState, MAX_NAME_LENGTH, server::Name},
    ps1_memory::GAMEPAD_BASE,
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

/// Handles `ServerMessage::Name`. Updates the name buffer for other
/// players. If the name starts with a null byte (player left) or the
/// current state is pre-race, forces SQUARE on that player's gamepad
/// to remove them from the lobby.
pub fn handle(ctr: &OnlineCtrSnapshot, state: &mut GameState, message: Name) -> Vec<Effect> {
    let driver_id = state.connection.driver_id;
    let slot = if message.client_id < driver_id {
        (message.client_id + 1) as usize
    } else {
        message.client_id as usize
    };
    if message.client_id != driver_id {
        let mut name_data = [0u8; MAX_NAME_LENGTH + 1];
        name_data[..MAX_NAME_LENGTH].copy_from_slice(&message.username[..MAX_NAME_LENGTH]);
        name_data[MAX_NAME_LENGTH - 1] = 0;

        let mut effects: Vec<Effect> = vec![
            Effect::SetDriverCount(message.client_count),
            Effect::SetNameBuffer {
                slot,
                data: name_data,
            },
        ];

        // handle disconnection - force SQUARE if name starts with 0
        if message.username[0] == 0 || ctr.current_state <= ClientState::LobbyWaitForLoading as i32
        {
            let gp_addr = GAMEPAD_BASE + (slot as u32 * 0x50);
            effects.push(Effect::WriteU32(gp_addr, 0x20));
            effects.push(Effect::WriteU32(gp_addr + 0x4, 0));
            effects.push(Effect::WriteU32(gp_addr + 0x8, 0));
            effects.push(Effect::WriteU32(gp_addr + 0xC, 0x20));
        }

        return effects;
    }

    vec![]
}

use crate::{
    effect::Effect,
    protocol::{ClientState, MAX_NAME_LENGTH, server::Name},
    ps1_memory::GAMEPAD_BASE,
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

pub fn handle(ctr: &OnlineCtrSnapshot, _state: &mut GameState, message: Name) -> Vec<Effect> {
    let driver_id = ctr.driver_id;
    if message.client_id != driver_id {
        let slot = if message.client_id < driver_id {
            (message.client_id + 1) as usize
        } else {
            message.client_id as usize
        };

        let mut name_data = [0u8; MAX_NAME_LENGTH + 1];
        for i in 0..MAX_NAME_LENGTH {
            name_data[i] = message.username[i];
        }
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

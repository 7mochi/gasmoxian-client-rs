use crate::{
    protocol::{ClientState, MAX_NAME_LENGTH, server::Name},
    ps1_memory::{GAMEPAD_BASE, Ps1Memory},
};

pub fn handle(ps1_memory: &mut Ps1Memory, message: Name) -> anyhow::Result<()> {
    let driver_id = ps1_memory.online_ctr().driver_id;
    if message.client_id != driver_id {
        let slot = if message.client_id < driver_id {
            message.client_id + 1
        } else {
            message.client_id
        } as usize;

        ps1_memory.online_ctr_mut().driver_count = message.client_count;
        for i in 0..MAX_NAME_LENGTH {
            ps1_memory.online_ctr_mut().name_buffer[slot][i] = message.username[i];
        }
        ps1_memory.online_ctr_mut().name_buffer[slot][MAX_NAME_LENGTH - 1] = 0;

        // handle disconnection - force SQUARE if name starts with 0
        if message.username[0] == 0
            || ps1_memory.online_ctr().current_state <= ClientState::LobbyWaitForLoading as i32
        {
            let gamepad_address = GAMEPAD_BASE + (slot as u32 * 0x50);
            ps1_memory.write_u32(gamepad_address, 0x20)?;
            ps1_memory.write_u32(gamepad_address + 0x4, 0)?;
            ps1_memory.write_u32(gamepad_address + 0x8, 0)?;
            ps1_memory.write_u32(gamepad_address + 0xC, 0x20)?;
        }
    }

    Ok(())
}

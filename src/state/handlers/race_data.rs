use crate::{
    protocol::{ClientState, server::Kart},
    ps1_memory::{GAMEPAD_BASE, LOADING_STAGE, PSX_POINTER, Ps1Memory},
    state::GameState,
};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    state: &mut GameState,
    message: Kart,
) -> anyhow::Result<()> {
    if ps1_memory.online_ctr().current_state < ClientState::GameWaitForRace as i32 {
        return Ok(());
    }
    if ps1_memory.read_u32(LOADING_STAGE)? != 0xFFFFFFFF {
        return Ok(());
    }

    let driver_id = ps1_memory.online_ctr().driver_id;
    if message.client_id == driver_id {
        return Ok(());
    }
    let slot = if message.client_id < driver_id {
        message.client_id + 1
    } else {
        message.client_id
    } as usize;

    let mut current_button = message.button_hold as u32;
    if (current_button & 0x40) != 0 {
        current_button &= !0x40;
        current_button |= 0x400;
    }
    if (current_button & 0x80) != 0 {
        current_button &= !0x80;
        current_button |= 0x800;
    }

    let previous_button = state.previous.buttons[slot] as u32;
    state.previous.buttons[slot] = current_button as i32;

    let gamepad_address = GAMEPAD_BASE + (slot as u32 * 0x50);
    ps1_memory.write_u32(gamepad_address, current_button)?;
    ps1_memory.write_u32(gamepad_address + 0x4, !previous_button & current_button)?;
    ps1_memory.write_u32(gamepad_address + 0x8, previous_button & !current_button)?;
    ps1_memory.write_u32(gamepad_address + 0xC, previous_button)?;

    let psx_pointer = ps1_memory.read_u32(PSX_POINTER + (slot as u32 * 4))? & 0xFFFFFF;
    ps1_memory.write_u32(
        psx_pointer + 0x2d4,
        ((message.position_x as i32) * 256) as u32,
    )?;
    ps1_memory.write_u32(
        psx_pointer + 0x2d8,
        ((message.position_y as i32) * 256) as u32,
    )?;
    ps1_memory.write_u32(
        psx_pointer + 0x2dc,
        ((message.position_z as i32) * 256) as u32,
    )?;

    if message.reserves {
        ps1_memory.write_u16(psx_pointer + 0x3e2, 200)?;
    }
    ps1_memory.write_u16(psx_pointer + 0x30, message.wumpa as u16)?;

    Ok(())
}

use crate::{
    effect::Effect,
    protocol::{ClientState, server::Kart},
    ps1_memory::GAMEPAD_BASE,
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

pub fn handle(ctr: &OnlineCtrSnapshot, state: &mut GameState, message: Kart) -> Vec<Effect> {
    if ctr.current_state < ClientState::GameWaitForRace as i32 {
        return vec![];
    }
    if ctr.loading_stage != 0xFFFFFFFF {
        return vec![];
    }

    let driver_id = ctr.driver_id;
    if message.client_id == driver_id {
        return vec![];
    }
    let slot = if message.client_id < driver_id {
        (message.client_id + 1) as usize
    } else {
        message.client_id as usize
    };

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

    // race_data reads the other players' PSX_POINTER offsets dynamically,
    // we need to read the slot psx_pointer: PSX_POINTER + (slot * 4)
    let mut effects = vec![
        Effect::WriteU32(gamepad_address, current_button),
        Effect::WriteU32(gamepad_address + 0x4, !previous_button & current_button),
        Effect::WriteU32(gamepad_address + 0x8, previous_button & !current_button),
        Effect::WriteU32(gamepad_address + 0xC, previous_button),
    ];

    // Cada jugador tiene su propio psx_pointer en slot_psx_pointers[slot]
    let slot_ptr = ctr.slot_psx_pointers[slot];
    effects.push(Effect::WriteU32(
        slot_ptr + 0x2d4,
        ((message.position_x as i32) * 256) as u32,
    ));
    effects.push(Effect::WriteU32(
        slot_ptr + 0x2d8,
        ((message.position_y as i32) * 256) as u32,
    ));
    effects.push(Effect::WriteU32(
        slot_ptr + 0x2dc,
        ((message.position_z as i32) * 256) as u32,
    ));

    if message.reserves {
        effects.push(Effect::WriteU16(slot_ptr + 0x3e2, 200));
    }
    effects.push(Effect::WriteU16(slot_ptr + 0x30, message.wumpa as u16));

    effects
}

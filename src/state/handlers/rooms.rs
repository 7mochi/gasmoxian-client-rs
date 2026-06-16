use crate::{
    console,
    protocol::{ClientState, GASMOXIAN_VERSION, server::Rooms},
    ps1_memory::Ps1Memory,
    state::GameState,
};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    _state: &mut GameState,
    message: Rooms,
) -> anyhow::Result<()> {
    console::debug(format!(
        "Client server: {}, Server version: {}, rooms: {}, client_count: {:?}",
        GASMOXIAN_VERSION, message.version, message.room_count, message.client_count
    ));

    ps1_memory.online_ctr_mut().pc_version = GASMOXIAN_VERSION as i32;
    ps1_memory.online_ctr_mut().server_version = message.version as i32;

    if message.version != GASMOXIAN_VERSION as u16 {
        console::err(format!(
            "Version mismatch! Client version: {}, Server version: {}",
            GASMOXIAN_VERSION, message.version
        ));

        ps1_memory.online_ctr_mut().current_state = ClientState::LaunchError as i32;
        return Ok(());
    }

    if ps1_memory.online_ctr().psx_version != GASMOXIAN_VERSION as i32 {
        console::err(format!(
            "PS1 version mismatch! Client version: {}, PS1 version: {}",
            GASMOXIAN_VERSION,
            ps1_memory.online_ctr().psx_version
        ));

        ps1_memory.online_ctr_mut().current_state = ClientState::LaunchError as i32;
        return Ok(());
    }

    ps1_memory.online_ctr_mut().server_lock_in2 = 0;
    ps1_memory.online_ctr_mut().room_count = message.room_count;

    for i in 0..16 {
        ps1_memory.online_ctr_mut().client_count[i] = message.client_count[i] as i8;
    }

    Ok(())
}

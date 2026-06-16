use crate::{
    console, enet::EnetClient, protocol::ClientState, ps1_memory::Ps1Memory, state::GameState,
};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    net: &mut EnetClient,
    state: &mut GameState,
) -> anyhow::Result<()> {
    console::err("Wrong password, returning to room list.");
    net.disconnect_now();

    state.flags.lock_engine_and_character = false;
    state.flags.password_sent = false;

    ps1_memory.online_ctr_mut().room_type = 0;
    ps1_memory.online_ctr_mut().room_type_locked = 0;
    ps1_memory.online_ctr_mut().auto_retry_join_room_index = -1;
    ps1_memory.online_ctr_mut().current_state = ClientState::LaunchPickRoom as i32;
    for i in 0..8 {
        ps1_memory.online_ctr_mut().password_entered[i] = 0;
        ps1_memory.online_ctr_mut().room_password_sequence[i] = 0;
    }

    Ok(())
}

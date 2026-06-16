use crate::{
    console,
    protocol::{ClientState, server::RoomType},
    ps1_memory::Ps1Memory,
    state::GameState,
};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    state: &mut GameState,
    message: RoomType,
) -> anyhow::Result<()> {
    ps1_memory.online_ctr_mut().room_type = message.room_type;
    console::debug(format!("Room type set to {}", message.room_type));

    if message.room_type == 2
        && ps1_memory.online_ctr().room_type_locked == 0
        && ps1_memory.online_ctr().current_state == ClientState::LaunchPickRoom as i32
    {
        console::debug(format!(
            "Room type is password protected, waiting for password input.",
        ));

        state.race.flags.password_sent = false;

        ps1_memory.online_ctr_mut().current_state = ClientState::LaunchEnterPassword as i32;
    }

    Ok(())
}

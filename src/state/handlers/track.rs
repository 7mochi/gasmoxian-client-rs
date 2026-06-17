use anyhow::Ok;

use crate::{
    console,
    protocol::{ClientState, server::Track},
    ps1_memory::Ps1Memory,
    state::GameState,
};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    state: &mut GameState,
    message: Track,
) -> anyhow::Result<()> {
    let num_laps = if message.lap_id >= 4 && message.lap_id <= 15 {
        let lap_values = [10, 15, 20, 25, 30, 35, 40, 50, 69, 80, 90, 127];
        lap_values[(message.lap_id - 4) as usize]
    } else {
        (message.lap_id * 2) + 1
    };

    console::debug(format!(
        "Setting track to {}, with {} laps",
        message.track_id, num_laps
    ));

    ps1_memory.write_u8(0x80096b20 + 0x1d33, num_laps)?;
    ps1_memory.online_ctr_mut().level_id = message.track_id;
    ps1_memory.online_ctr_mut().current_state = ClientState::LobbySpecialPick as i32;

    state.race.extra_laps = if message.lap_id >= 8 { 1 } else { 0 };

    Ok(())
}

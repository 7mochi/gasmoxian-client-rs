use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    protocol::server::EndRace,
    ps1_memory::{GAMEPAD_BASE, Ps1Memory},
    state::GameState,
};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    state: &mut GameState,
    message: EndRace,
) -> anyhow::Result<()> {
    let driver_id = ps1_memory.online_ctr().driver_id;
    if message.client_id == driver_id {
        return Ok(());
    }
    let slot = if message.client_id < driver_id {
        message.client_id + 1
    } else {
        message.client_id
    } as usize;

    if state.race.square_delay[slot] == 0 {
        state.race.square_delay[slot] = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX_EPOCH")
        .as_secs();
    if now - state.race.square_delay[slot] >= 3 {
        let gp_addr = GAMEPAD_BASE + (slot as u32 * 0x50);

        ps1_memory.write_u32(gp_addr, 0x20)?;
        ps1_memory.write_u32(gp_addr + 0x4, 0)?;
        ps1_memory.write_u32(gp_addr + 0x8, 0)?;
        ps1_memory.write_u32(gp_addr + 0xC, 0x20)?;
    }

    let ended = ps1_memory.online_ctr().drivers_ended_count as usize;

    ps1_memory.online_ctr_mut().race_stats[ended].slot = slot as i32;
    ps1_memory.online_ctr_mut().race_stats[ended].final_time = message.course_time;
    ps1_memory.online_ctr_mut().race_stats[ended].best_lap = message.lap_time;
    ps1_memory.online_ctr_mut().drivers_ended_count += 1;

    Ok(())
}

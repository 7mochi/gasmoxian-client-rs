use crate::{
    effect::Effect,
    protocol::{RaceStats, server::EndRace},
    ps1_memory::GAMEPAD_BASE,
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

pub fn handle(ctr: &OnlineCtrSnapshot, state: &mut GameState, message: EndRace) -> Vec<Effect> {
    let driver_id = ctr.driver_id;
    if message.client_id == driver_id {
        return vec![];
    }
    let slot = if message.client_id < driver_id {
        (message.client_id + 1) as usize
    } else {
        message.client_id as usize
    };

    let mut effects: Vec<Effect> = Vec::new();

    if state.race.square_delay[slot] == 0 {
        state.race.square_delay[slot] = ctr.now_secs as u64;
    }

    if (ctr.now_secs as u64) - state.race.square_delay[slot] >= 3 {
        let gp_addr = GAMEPAD_BASE + (slot as u32 * 0x50);
        effects.push(Effect::WriteU32(gp_addr, 0x20));
        effects.push(Effect::WriteU32(gp_addr + 0x4, 0));
        effects.push(Effect::WriteU32(gp_addr + 0x8, 0));
        effects.push(Effect::WriteU32(gp_addr + 0xC, 0x20));
    }

    let ended = ctr.drivers_ended_count as usize;
    effects.push(Effect::WriteRaceStats {
        slot: ended,
        stats: RaceStats {
            slot: slot as i32,
            final_time: message.course_time,
            best_lap: message.lap_time,
        },
    });
    effects.push(Effect::SetDriversEndedCount(ctr.drivers_ended_count + 1));

    effects
}

use crate::{protocol::server::WarpClock, ps1_memory::Ps1Memory, state::GameState};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    state: &mut GameState,
    message: WarpClock,
) -> anyhow::Result<()> {
    state.previous.warpclock = Some(message.warp_clock as i32);
    if ps1_memory.online_ctr().warpclock != message.warp_clock {
        ps1_memory.online_ctr_mut().warpclock = message.warp_clock;
    }

    Ok(())
}

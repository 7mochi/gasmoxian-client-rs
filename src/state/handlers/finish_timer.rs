use crate::{protocol::server::FinishTimer, ps1_memory::Ps1Memory, state::GameState};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    state: &mut GameState,
    message: FinishTimer,
) -> anyhow::Result<()> {
    let current_timer = message.finish_timer as i32;

    if state.previous.finish_timer != Some(current_timer) {
        ps1_memory.online_ctr_mut().finish_race_timer = message.finish_timer;

        state.previous.finish_timer = Some(current_timer);
    }

    Ok(())
}

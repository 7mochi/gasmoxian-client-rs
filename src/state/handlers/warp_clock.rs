use crate::{
    effect::Effect, protocol::server::WarpClock, ps1_snapshot::OnlineCtrSnapshot, state::GameState,
};

/// Handles `ServerMessage::Warpclock`. Updates the warpclock in PS1
/// memory only when the value actually changed (prevents redundant writes).
pub fn handle(ctr: &OnlineCtrSnapshot, state: &mut GameState, message: WarpClock) -> Vec<Effect> {
    state.previous.warpclock = Some(message.warp_clock as i32);
    if ctr.warpclock != message.warp_clock {
        vec![Effect::SetWarpclock(message.warp_clock)]
    } else {
        vec![]
    }
}

use crate::{
    effect::Effect, protocol::server::WarpClock, ps1_snapshot::OnlineCtrSnapshot, state::GameState,
};

pub fn handle(ctr: &OnlineCtrSnapshot, state: &mut GameState, message: WarpClock) -> Vec<Effect> {
    state.previous.warpclock = Some(message.warp_clock as i32);
    if ctr.warpclock != message.warp_clock {
        vec![Effect::SetWarpclock(message.warp_clock)]
    } else {
        vec![]
    }
}

use crate::{
    effect::Effect, protocol::server::FinishTimer, ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

/// Handles `ServerMessage::FinishTimer`. Updates the finish countdown
/// timer in PS1 memory, ignoring duplicate values.
pub fn handle(
    _ctr: &OnlineCtrSnapshot,
    state: &mut GameState,
    message: FinishTimer,
) -> Vec<Effect> {
    let current_timer = message.finish_timer as i32;

    if state.previous.finish_timer != Some(current_timer) {
        state.previous.finish_timer = Some(current_timer);
        vec![Effect::SetFinishRaceTimer(message.finish_timer)]
    } else {
        vec![]
    }
}

use crate::{
    effect::Effect, protocol::server::Engine, ps1_snapshot::OnlineCtrSnapshot, state::GameState,
};

pub fn handle(ctr: &OnlineCtrSnapshot, _state: &mut GameState, message: Engine) -> Vec<Effect> {
    let driver_id = ctr.driver_id;
    if message.client_id != driver_id {
        let slot = if message.client_id < driver_id {
            message.client_id + 1
        } else {
            message.client_id
        };

        let engine = if message.engine_type > 3 {
            3
        } else {
            message.engine_type
        };

        return vec![
            Effect::SetEngineType {
                slot: slot as usize,
                value: engine as i8,
            },
            Effect::SetLockedInEngine {
                slot: message.client_id as usize,
                value: message.locked_in as i8,
            },
        ];
    }

    vec![]
}

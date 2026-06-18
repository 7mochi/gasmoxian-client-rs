use crate::{
    effect::Effect, protocol::server::Character, ps1_snapshot::OnlineCtrSnapshot, state::GameState,
};

/// Handles `ServerMessage::Character`. Writes other players' character
/// choices to PS1 character memory and updates the locked-in state.
pub fn handle(_ctr: &OnlineCtrSnapshot, state: &mut GameState, message: Character) -> Vec<Effect> {
    let driver_id = state.connection.driver_id;
    if message.client_id != driver_id {
        let slot = if message.client_id < driver_id {
            message.client_id + 1
        } else {
            message.client_id
        };

        return vec![
            Effect::WriteU16(0x80086e84 + (slot as u32 * 2), message.character_id as u16),
            Effect::SetLockedInCharacter {
                slot: message.client_id as usize,
                value: message.locked_in as i8,
            },
        ];
    }

    vec![]
}

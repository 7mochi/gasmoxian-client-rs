use crate::{
    effect::Effect,
    protocol::{ShootSlot, server::Weapon},
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

/// Handles `ServerMessage::Weapon`. Sets the shoot slot (`now = 1`)
/// for another player's weapon pickup.
pub fn handle(_ctr: &OnlineCtrSnapshot, state: &mut GameState, message: Weapon) -> Vec<Effect> {
    let driver_id = state.connection.driver_id;
    if message.client_id != driver_id {
        let slot = if message.client_id < driver_id {
            message.client_id + 1
        } else {
            message.client_id
        };

        return vec![Effect::SetShoot {
            slot: slot as usize,
            shoot: ShootSlot {
                now: 1,
                weapon: message.weapon,
                juiced: message.juiced as u8,
                flags: message.flags,
            },
        }];
    }

    vec![]
}

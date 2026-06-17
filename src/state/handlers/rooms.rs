use crate::{
    effect::Effect,
    protocol::{ClientState, GASMOXIAN_VERSION, server::Rooms},
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

pub fn handle(ctr: &OnlineCtrSnapshot, _state: &mut GameState, message: Rooms) -> Vec<Effect> {
    let mut effects = vec![
        Effect::LogDebug(format!(
            "Client server: {}, Server version: {}, rooms: {}, client_count: {:?}",
            GASMOXIAN_VERSION, message.version, message.room_count, message.client_count
        )),
        Effect::SetPcVersion(GASMOXIAN_VERSION as i32),
        Effect::SetServerVersion(message.version as i32),
    ];

    if message.version != GASMOXIAN_VERSION as u16 {
        effects.push(Effect::LogErr(format!(
            "Version mismatch! Client version: {}, Server version: {}",
            GASMOXIAN_VERSION, message.version
        )));
        effects.push(Effect::SetState(ClientState::LaunchError));
        return effects;
    }

    if ctr.psx_version != GASMOXIAN_VERSION as i32 {
        effects.push(Effect::LogErr(format!(
            "PS1 version mismatch! Client version: {}, PS1 version: {}",
            GASMOXIAN_VERSION, ctr.psx_version
        )));
        effects.push(Effect::SetState(ClientState::LaunchError));
        return effects;
    }

    effects.push(Effect::SetServerLockIn2(0));
    effects.push(Effect::SetRoomCount(message.room_count));

    let mut client_count = [0i8; 16];
    for i in 0..16 {
        client_count[i] = message.client_count[i] as i8;
    }
    effects.push(Effect::SetClientCount(client_count));

    effects
}

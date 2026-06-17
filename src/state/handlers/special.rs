use crate::{
    effect::Effect,
    protocol::{ClientState, Gamemode, server::Special},
    ps1_memory::CHEATS,
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

/// Handles `ServerMessage::Special`. Copies all 18 gamemode toggles
/// to PS1 memory, applies cheat bits for Icy Tracks and Retro Fueled,
/// and transitions to `LobbyCharacterPick`.
pub fn handle(ctr: &OnlineCtrSnapshot, _state: &mut GameState, message: Special) -> Vec<Effect> {
    // copy all gamemodes toggles
    let mut effects: Vec<Effect> = (0..18)
        .map(|i| Effect::SetGamemode {
            index: i,
            value: message.gamemodes[i],
        })
        .collect();
    effects.push(Effect::SetGamemode {
        index: Gamemode::Normal as usize,
        value: true,
    });

    // apply cheat effects
    let mut cheats = ctr.cheats;
    cheats &= !(0x100000 | 0x80000 | 0x400 | 0x400000 | 0x8000000 | 0x10000);

    if message.gamemodes[Gamemode::IcyTracks as usize] {
        cheats |= 0x80000;
    }
    if message.gamemodes[Gamemode::RetroFueled as usize] {
        cheats |= 0x100000;
    }
    effects.push(Effect::WriteU32(CHEATS, cheats));

    effects.push(Effect::SetState(ClientState::LobbyCharacterPick));

    effects
}

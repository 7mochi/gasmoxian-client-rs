use crate::{
    protocol::{ClientState, Gamemode, server::Special},
    ps1_memory::{CHEATS, Ps1Memory},
};

pub fn handle(ps1_memory: &mut Ps1Memory, message: Special) -> anyhow::Result<()> {
    // copy all gamemodes toggles
    for i in 0..18 {
        ps1_memory.online_ctr_mut().gamemodes[i] = message.gamemodes[i];
    }
    ps1_memory.online_ctr_mut().gamemodes[Gamemode::Normal as usize] = true;

    // apply cheat effects
    let mut cheats = ps1_memory.read_u32(CHEATS)?;
    cheats &= !(0x100000 | 0x80000 | 0x400 | 0x400000 | 0x8000000 | 0x10000);

    if ps1_memory.online_ctr_mut().gamemodes[Gamemode::IcyTracks as usize] {
        cheats |= 0x80000;
    }
    if ps1_memory.online_ctr_mut().gamemodes[Gamemode::RetroFueled as usize] {
        cheats |= 0x100000;
    }
    ps1_memory.write_u32(CHEATS, cheats)?;

    ps1_memory.online_ctr_mut().current_state = ClientState::LobbyCharacterPick as i32;

    Ok(())
}

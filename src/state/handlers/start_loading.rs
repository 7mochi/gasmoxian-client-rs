use crate::{protocol::ClientState, ps1_memory::Ps1Memory};

pub fn handle(ps1_memory: &mut Ps1Memory) -> anyhow::Result<()> {
    ps1_memory.online_ctr_mut().current_state = ClientState::LobbyStartLoading as i32;

    Ok(())
}

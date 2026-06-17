use crate::{protocol::server::Weapon, ps1_memory::Ps1Memory};

pub fn handle(ps1_memory: &mut Ps1Memory, message: Weapon) -> anyhow::Result<()> {
    let driver_id = ps1_memory.online_ctr().driver_id;
    if message.client_id != driver_id {
        let slot = if message.client_id < driver_id {
            message.client_id + 1
        } else {
            message.client_id
        };

        ps1_memory.online_ctr_mut().shoot[slot as usize].now = 1;
        ps1_memory.online_ctr_mut().shoot[slot as usize].weapon = message.weapon;
        ps1_memory.online_ctr_mut().shoot[slot as usize].juiced = message.juiced as u8;
        ps1_memory.online_ctr_mut().shoot[slot as usize].flags = message.flags;
    }

    Ok(())
}

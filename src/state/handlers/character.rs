use crate::{protocol::server::Character, ps1_memory::Ps1Memory};

pub fn handle(ps1_memory: &mut Ps1Memory, message: Character) -> anyhow::Result<()> {
    let driver_id = ps1_memory.online_ctr().driver_id;
    if message.client_id != driver_id {
        let slot = if message.client_id < driver_id {
            message.client_id + 1
        } else {
            message.client_id
        };

        ps1_memory.write_u16(0x80086e84 + (slot as u32 * 2), message.character_id as u16)?;

        ps1_memory.online_ctr_mut().locked_in_characters[message.client_id as usize] =
            message.locked_in as i8;
    }

    Ok(())
}

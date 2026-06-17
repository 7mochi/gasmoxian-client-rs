use crate::{protocol::server::Engine, ps1_memory::Ps1Memory};

pub fn handle(ps1_memory: &mut Ps1Memory, message: Engine) -> anyhow::Result<()> {
    let driver_id = ps1_memory.online_ctr().driver_id;
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

        ps1_memory.online_ctr_mut().engine_type[slot as usize] = engine as i8;
        ps1_memory.online_ctr_mut().locked_in_engines[message.client_id as usize] =
            message.locked_in as i8;
    }

    Ok(())
}

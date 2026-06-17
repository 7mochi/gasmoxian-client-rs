use deku::DekuContainerWrite;

use crate::protocol::ClientState;
use crate::protocol::client::Name;
use crate::{
    console,
    enet::EnetClient,
    protocol::{ClientMessage, MAX_NAME_LENGTH, MAX_NUM_PLAYERS, server::ClientStatus},
    ps1_memory::Ps1Memory,
    state::GameState,
};

pub fn handle(
    ps1_memory: &mut Ps1Memory,
    net: &mut EnetClient,
    state: &mut GameState,
    message: ClientStatus,
) -> anyhow::Result<()> {
    if ps1_memory.online_ctr().server_room == 15 {
        console::info("Easter egg unlocked: Saffi fire unlocked in this room!");
    }

    state.race.flags.password_sent = false;
    state.race.flags.lock_engine_and_character = false;
    state.race.flags.packet_already_sent = false;
    state.race.flags.sent_warpclock = false;
    state.previous.warpclock = Some(-1);
    state.previous.special = Some(-1);
    state.previous.finish_timer = Some(-1);
    state.race.extra_laps = 0;
    state.race.square_delay = [0; MAX_NUM_PLAYERS];

    ps1_memory.online_ctr_mut().driver_id = message.client_id;
    ps1_memory.online_ctr_mut().driver_count = message.client_count;
    ps1_memory.online_ctr_mut().locked_in_lap = 0;
    ps1_memory.online_ctr_mut().locked_in_level = 0;
    ps1_memory.online_ctr_mut().locked_in_engine = 0;
    ps1_memory.online_ctr_mut().locked_in_special = 0;
    ps1_memory.online_ctr_mut().lap_id = 0;
    ps1_memory.online_ctr_mut().special = 0;
    ps1_memory.online_ctr_mut().level_id = 0;
    ps1_memory.online_ctr_mut().locked_in_character = 0;
    ps1_memory.online_ctr_mut().drivers_ended_count = 0;
    ps1_memory.online_ctr_mut().finish_race_timer = 0;
    ps1_memory.online_ctr_mut().warpclock = 0;

    for i in 0..MAX_NUM_PLAYERS {
        ps1_memory.online_ctr_mut().locked_in_characters[i] = 0;
        ps1_memory.online_ctr_mut().locked_in_engines[i] = 0;
        ps1_memory.online_ctr_mut().race_stats[i].slot = 0;
        ps1_memory.online_ctr_mut().race_stats[i].final_time = 0;
        ps1_memory.online_ctr_mut().race_stats[i].best_lap = 0;
        for j in 0..MAX_NAME_LENGTH {
            ps1_memory.online_ctr_mut().name_buffer[i][j] = 0;
        }
    }

    for i in 0..8 {
        ps1_memory.online_ctr_mut().password_entered[i] = 0;
        ps1_memory.online_ctr_mut().room_password_sequence[i] = 0;
    }

    // send name to the server
    let username_buffer = state.lobby.username.as_bytes();
    let name_len = username_buffer.len().min(MAX_NAME_LENGTH);
    ps1_memory.online_ctr_mut().name_buffer[0][..name_len]
        .copy_from_slice(&username_buffer[..name_len]);
    ps1_memory.online_ctr_mut().name_buffer[0][MAX_NAME_LENGTH] = 0;

    let mut username = [0u8; 12];
    username[..name_len].copy_from_slice(&username_buffer[..name_len]);

    let client_message = Name::new(username)
        .to_bytes()
        .expect("Failed to serialize name message");

    net.send_reliable(&client_message)
        .expect("Failed to send name message");

    ps1_memory.online_ctr_mut().current_state = ClientState::LobbyAssignRole as i32;

    Ok(())
}

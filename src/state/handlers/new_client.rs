use deku::DekuContainerWrite;

use crate::protocol::ClientState;
use crate::protocol::client::Name;
use crate::{
    effect::Effect,
    protocol::{MAX_NAME_LENGTH, MAX_NUM_PLAYERS, server::ClientStatus},
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

pub fn handle(
    ctr: &OnlineCtrSnapshot,
    state: &mut GameState,
    message: ClientStatus,
) -> Vec<Effect> {
    let mut effects: Vec<Effect> = Vec::new();

    if ctr.server_room == 15 {
        effects.push(Effect::LogInfo(
            "Easter egg unlocked: Saffi fire unlocked in this room!".into(),
        ));
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

    effects.push(Effect::SetDriverId(message.client_id));
    effects.push(Effect::SetDriverCount(message.client_count));
    effects.push(Effect::SetLockedInLap(0));
    effects.push(Effect::SetLockedInLevel(0));
    effects.push(Effect::SetLockedInEngineByte(0));
    effects.push(Effect::SetLockedInSpecial(0));
    effects.push(Effect::SetLapId(0));
    effects.push(Effect::SetSpecial(0));
    effects.push(Effect::SetLevelId(0));
    effects.push(Effect::SetLockedInCharacterByte(0));
    effects.push(Effect::SetDriversEndedCount(0));
    effects.push(Effect::SetFinishRaceTimer(0));
    effects.push(Effect::SetWarpclock(0));

    for i in 0..MAX_NUM_PLAYERS {
        effects.push(Effect::SetLockedInCharacter { slot: i, value: 0 });
        effects.push(Effect::SetLockedInEngine { slot: i, value: 0 });
        effects.push(Effect::WriteRaceStats {
            slot: i,
            stats: crate::protocol::RaceStats {
                slot: 0,
                final_time: 0,
                best_lap: 0,
            },
        });
        let name_data = [0u8; MAX_NAME_LENGTH + 1];
        effects.push(Effect::SetNameBuffer {
            slot: i,
            data: name_data,
        });
    }

    effects.push(Effect::SetPasswordEntered([0u8; 8]));
    effects.push(Effect::SetRoomPasswordSequence([0u8; 8]));

    // send name to the server
    let username_buffer = state.lobby.username.as_bytes();
    let name_len = username_buffer.len().min(MAX_NAME_LENGTH);
    let mut name_data = [0u8; MAX_NAME_LENGTH + 1];
    name_data[..name_len].copy_from_slice(&username_buffer[..name_len]);
    effects.push(Effect::SetNameBuffer {
        slot: 0,
        data: name_data,
    });

    let mut username = [0u8; 12];
    username[..name_len].copy_from_slice(&username_buffer[..name_len]);

    let client_message = Name::new(username)
        .to_bytes()
        .expect("Failed to serialize name message");

    effects.push(Effect::SendReliable(client_message));
    effects.push(Effect::SetState(ClientState::LobbyAssignRole));

    effects
}

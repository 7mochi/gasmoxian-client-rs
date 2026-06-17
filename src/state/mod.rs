use std::{net::SocketAddr, time::Duration};

use deku::DekuContainerWrite;
use rusty_enet::Event::{self};

use crate::{
    effect::Effect,
    enet::EnetClient,
    protocol::{
        ClientState,
        Gamemode::{self},
        MAX_NUM_PLAYERS, RaceStats,
        client::{
            Character, EndRace, Engine, FinishTimer, Kart, Password, Room, RoomType,
            RoomTypePassword, Special, StartRace, Track, WarpClock, Weapon,
        },
    },
    ps1_memory::{LOBBY_LEVEL_ID, Ps1Memory},
    ps1_snapshot::OnlineCtrSnapshot,
    server::SERVERS,
};

pub mod handlers;

const PREVIOUS_BUTTONS_SIZE: usize = 8;
const AFK_TIMEOUT: f64 = 80.0;

#[derive(Debug, Default)]
pub struct PlayerSelection {
    pub character_id: Option<i32>,
    pub is_character_locked: bool,
    pub engine_type: Option<i32>,
    pub is_engine_locked: bool,
}

#[derive(Debug, Default)]
pub struct RaceFlags {
    pub password_sent: bool,
    pub lock_engine_and_character: bool,
    pub sent_warpclock: bool,
    pub sent_start_race: bool,
    pub sent_end_race: bool,
    pub packet_already_sent: bool,
}

#[derive(Debug, Default)]
pub struct Connection {
    pub attempt: i32,
    pub server_addr: Option<SocketAddr>,
    pub static_server_id: i32,
    pub static_room_id: i32,
}

#[derive(Debug, Default)]
pub struct Race {
    pub count_frame: i32,
    pub time_start: f64,
    pub warpclock_delay: f64,
    pub square_delay: [u64; MAX_NUM_PLAYERS],
    // TODO: change this when we figure out what the timers are representing
    pub timers: [f64; 2],
    pub extra_laps: i32,
    pub flags: RaceFlags,
}

#[derive(Debug, Default)]
pub struct PreviousInput {
    pub warpclock: Option<i32>,
    pub special: Option<i32>,
    pub finish_timer: Option<i32>,
    // TODO: change this when we figure out what buttons the indexes of the array are representing
    pub buttons: [i32; PREVIOUS_BUTTONS_SIZE],
}

#[derive(Debug, Default)]
pub struct Lobby {
    pub username: String,
    pub required_players: i32,
    pub disconnected_players: i32,
    pub active_players: i32,
}

#[derive(Debug)]
pub struct GameState {
    pub connection: Connection,
    pub race: Race,
    pub previous: PreviousInput,
    pub lobby: Lobby,
    pub previous_selection: PlayerSelection,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            connection: Connection::default(),
            race: Race::default(),
            previous: PreviousInput::default(),
            lobby: Lobby::default(),
            previous_selection: PlayerSelection::default(),
        }
    }
}

pub fn process_network_messages(
    ctr: &OnlineCtrSnapshot,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) -> Vec<Effect> {
    let net = match net {
        Some(n) => n,
        None => return vec![],
    };
    let mut effects: Vec<Effect> = Vec::new();
    while let Ok(Some(event)) = net.poll() {
        match event {
            Event::Receive { packet, .. } => {
                effects.extend(handlers::process_receive_event(ctr, state, packet.data()));
            }
            Event::Disconnect { .. } => {
                effects.push(Effect::LogErr(
                    "Connection Dropped (Server Full or Server Offline)...".into(),
                ));

                state.race.flags.password_sent = false;

                effects.push(Effect::SetStateRaw(-1));
            }
            _ => {}
        }
    }
    effects
}

pub fn frame_stall(ps1_memory: &mut Ps1Memory) {
    while ps1_memory.online_ctr().ready_to_send == 0 {
        std::thread::sleep(Duration::from_micros(1));
    }

    ps1_memory.online_ctr_mut().ready_to_send = 0;
}

pub fn disconnect(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    if (ctr.gamepad_hold & 0x2000) != 0 {
        state.race.flags.lock_engine_and_character = false;

        vec![
            Effect::LogInfo("Disconnected from server (the player pressed DSELECT)".into()),
            Effect::DisconnectNow,
            Effect::SetAutoRetryRoomIndex(-1),
            Effect::SetRoomType(0),
            Effect::SetRoomTypeLocked(0),
            Effect::SetStateRaw(-1),
        ]
    } else {
        vec![]
    }
}

pub fn afk_timer(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    if !state.race.flags.lock_engine_and_character {
        state.race.time_start = 0.0;
        return vec![];
    }

    if state.race.time_start == 0.0 {
        state.race.time_start = ctr.now_secs;
        return vec![Effect::LogDebug(format!(
            "AFK timer started (timeout: {}s)",
            AFK_TIMEOUT
        ))];
    }

    if (ctr.now_secs - state.race.time_start) >= AFK_TIMEOUT {
        if !state.race.flags.lock_engine_and_character {
            state.race.time_start = 0.0;
            return vec![];
        }
        state.race.flags.lock_engine_and_character = false;
        state.race.time_start = 0.0;
        vec![
            Effect::LogErr("Kicked for AFK".into()),
            Effect::DisconnectNow,
            Effect::SetRoomType(0),
            Effect::SetRoomTypeLocked(0),
            Effect::SetStateRaw(-1),
        ]
    } else {
        vec![]
    }
}

pub fn launch_enter_pid(ctr: &OnlineCtrSnapshot) -> Vec<Effect> {
    if ctr.is_booted_ps1 == 0 {
        return vec![];
    }
    vec![
        Effect::LogDebug(format!(
            "PS1 booted (psx_version: {}, pc_version: {})",
            ctr.psx_version, ctr.pc_version
        )),
        Effect::LogOk("Connected to DuckStation".into()),
        Effect::LogInfo("Waiting to connect to a server...".into()),
        Effect::SetState(ClientState::LaunchPickServer),
    ]
}

pub fn launch_pick_server(
    ctr: &OnlineCtrSnapshot,
    state: &mut GameState,
    server_addrs: &[Option<SocketAddr>],
) -> Vec<Effect> {
    // quit if disconnected but not loaded, back into the selection screen yet

    // must be in cutscene level to see country selector
    if ctr.cutscene_level_id != LOBBY_LEVEL_ID {
        return vec![];
    }

    // quit if in loading screen (force-reconnect)
    if ctr.loading_stage != 0xFFFFFFFF {
        return vec![];
    }

    // return now if the server selection hasn't been selected yet
    if ctr.server_lock_in1 == 0 {
        return vec![];
    }

    let server_country = ctr.server_country as usize;

    // now selecting country
    let server = &SERVERS[server_country];

    if let Some(addr) = server_addrs.get(server_country).and_then(|o| *o) {
        state.connection.server_addr = Some(addr);
        state.connection.static_server_id = server_country as i32;
        vec![
            Effect::LogOk("Ready for server selection.".into()),
            Effect::SetClientBusy(1),
            Effect::LogInfo(format!("Ready to connect to {}", server.endpoint)),
        ]
    } else {
        vec![]
    }
}

pub fn launch_error() -> Vec<Effect> {
    // Version mismatch or other connection error — wait for user to return to menu.
    vec![]
}

pub fn launch_enter_password(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    if state.race.flags.password_sent {
        return vec![];
    }

    // keep alive via enet ping to prevent timeout
    state.race.count_frame += 1;
    let mut effects: Vec<Effect> = Vec::new();
    if state.race.count_frame >= 60 {
        state.race.count_frame = 0;
        effects.push(Effect::Ping);
    }

    if ctr.password_entered[7] == 0 {
        return effects;
    }

    let mut sequence = [0u8; 8];
    sequence.copy_from_slice(&ctr.room_password_sequence);

    let client_message = Password::new(sequence)
        .to_bytes()
        .expect("Failed to serialize password message");

    effects.push(Effect::SendReliable(client_message));
    state.race.flags.password_sent = true;
    effects
}

pub fn lobby_assign_role(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    state.connection.attempt = 0;
    state.race.count_frame = 0;

    // guest: do nothing
    if ctr.driver_id > 0 {
        return vec![];
    }

    if ctr.room_type_locked == 0 {
        return vec![];
    }

    let room_type = ctr.room_type;
    let room_type_locked = ctr.room_type_locked;

    if room_type == 2 {
        let mut sequence = [0u8; 8];
        sequence.copy_from_slice(&ctr.room_password_sequence);

        let client_message = RoomTypePassword::new(room_type, room_type_locked, sequence)
            .to_bytes()
            .expect("Failed to serialize room type password message");

        vec![Effect::SendReliable(client_message)]
    } else {
        let client_message = RoomType::new(room_type, room_type_locked)
            .to_bytes()
            .expect("Failed to serialize room type message");

        vec![Effect::SendReliable(client_message)]
    }
}

pub fn lobby_character_pick(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    let character_id = ctr.character_id as i32;
    let locked_in = ctr.locked_in_characters[ctr.driver_id as usize] as i32;

    let previous_selection = &mut state.previous_selection;
    let has_changed = previous_selection.character_id != Some(character_id)
        || previous_selection.is_character_locked != (locked_in != 0);

    let mut effects: Vec<Effect> = Vec::new();

    if has_changed {
        previous_selection.character_id = Some(character_id);
        previous_selection.is_character_locked = locked_in != 0;

        let client_message = Character::new(character_id as u8, locked_in != 0)
            .to_bytes()
            .expect("Failed to serialize character message");

        effects.push(Effect::SendReliable(client_message));
    }

    if locked_in != 0 {
        effects.push(Effect::SetState(ClientState::LobbyEnginePick));
    }
    effects
}

pub fn lobby_engine_pick(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    let engine_type = ctr.engine_type[0] as i32;
    let locked_in = ctr.locked_in_engines[ctr.driver_id as usize] as i32;

    let previous_selection = &mut state.previous_selection;
    let has_changed = previous_selection.engine_type != Some(engine_type)
        || previous_selection.is_engine_locked != (locked_in != 0);

    let mut effects: Vec<Effect> = Vec::new();

    if has_changed {
        previous_selection.engine_type = Some(engine_type);
        previous_selection.is_engine_locked = locked_in != 0;

        let client_message = Engine::new(engine_type as u8, locked_in != 0)
            .to_bytes()
            .expect("Failed to serialize engine message");

        effects.push(Effect::SendReliable(client_message));
    }

    if locked_in != 0 {
        state.race.flags.lock_engine_and_character = false;

        effects.push(Effect::SetState(ClientState::LobbyWaitForLoading));
    }
    effects
}

pub fn lobby_special_pick(ctr: &OnlineCtrSnapshot, _state: &mut GameState) -> Vec<Effect> {
    if ctr.locked_in_special == 0 {
        return vec![];
    }

    let mut gamemodes = [false; 18];
    gamemodes.copy_from_slice(&ctr.gamemodes);

    // always ensure GameMode::Normal is enabled
    gamemodes[Gamemode::Normal as usize] = true;

    let client_message = Special::new(gamemodes)
        .to_bytes()
        .expect("Failed to serialize special message");

    vec![
        Effect::SendReliable(client_message),
        Effect::SetState(ClientState::LobbyCharacterPick),
    ]
}

pub fn lobby_host_track_pick(ctr: &OnlineCtrSnapshot, _state: &mut GameState) -> Vec<Effect> {
    // locked_in_lap gets set after locked_in_level already sets
    if ctr.locked_in_lap == 0 {
        return vec![];
    }

    let lap_id = ctr.lap_id;
    let track_id = ctr.level_id;

    let num_laps = if (4..=15).contains(&lap_id) {
        let lap_values = [10, 15, 20, 25, 30, 35, 40, 50, 69, 80, 90, 127];
        lap_values[(lap_id - 4) as usize]
    } else {
        (lap_id * 2) + 1
    };

    let client_message = Track::new(track_id, lap_id)
        .to_bytes()
        .expect("Failed to serialize track message");

    vec![
        Effect::WriteU8(0x80096b20 + 0x1d33, num_laps),
        Effect::SendReliable(client_message),
        Effect::SetState(ClientState::LobbySpecialPick),
    ]
}

pub fn lobby_guest_track_wait(state: &mut GameState) -> Vec<Effect> {
    state.previous_selection.character_id = None;
    state.previous_selection.is_character_locked = false;
    state.previous_selection.engine_type = None;
    state.previous_selection.is_engine_locked = false;
    vec![]
}

pub fn lobby_wait_for_loading() -> Vec<Effect> {
    // if recv message to start loading, change state to StartLoading, this check happens in ProcessNewMessages
    vec![]
}

pub fn lobby_start_loading(state: &mut GameState) -> Vec<Effect> {
    state.race.flags.sent_start_race = false;
    state.race.flags.sent_end_race = false;
    vec![Effect::SetFinishRaceTimer(0)]
}

pub fn launch_pick_room(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    let mut effects: Vec<Effect> = Vec::new();

    state.race.count_frame += 1;

    // room not updating bug still happens if the number is not 60, i didnt tried 30 anyways
    if state.race.count_frame == 60 {
        state.race.count_frame = 0;

        // send junk data, this triggers server response
        let client_message = Room::new(0xFF)
            .to_bytes()
            .expect("Failed to serialize join room message");

        effects.push(Effect::SendReliable(client_message));
    }

    // wait for room to be chosen
    if ctr.server_lock_in2 == 0 {
        state.connection.attempt = 0;
        return effects;
    }

    // dont send ClientMsg::JoinRoom twice
    if state.connection.attempt == 1 {
        return effects;
    }
    state.connection.attempt = 1;

    let room = ctr.server_room;
    effects.push(Effect::SetAutoRetryRoomIndex(-1));

    let client_message = Room::new(room)
        .to_bytes()
        .expect("Failed to serialize join room message");

    effects.push(Effect::SendReliable(client_message));
    effects
}

fn send_everything(ctr: &OnlineCtrSnapshot) -> Vec<Effect> {
    // position
    let hold_raw = ctr.gamepad_hold;

    // lossless compression, bottom byte is never used,
    // cause psx renders with 3 bytes, and top byte
    // is never used due to world scale (just pure luck)

    // ignore Circle/L2
    let mut hold = hold_raw & !0xC0;

    // put L1/R1 into one byte
    if (hold & 0x400) != 0 {
        hold |= 0x40;
    }
    if (hold & 0x800) != 0 {
        hold |= 0x80;
    }

    let kart_msg = Kart::new(
        ctr.kart_wumpa,
        ctr.kart_reserves > 200,
        (ctr.kart_angle & 0x1f) as u8,
        (ctr.kart_angle >> 5) as u8,
        hold as u8,
        ctr.kart_position_x,
        ctr.kart_position_y,
        ctr.kart_position_z,
    )
    .to_bytes()
    .expect("Failed to serialize kart message");

    let mut effects = vec![Effect::SendUnsequenced(kart_msg)];

    if ctr.shoot[0].now != 0 {
        let weapon_msg = Weapon::new(
            ctr.shoot[0].juiced != 0,
            ctr.shoot[0].flags,
            ctr.shoot[0].weapon,
        )
        .to_bytes()
        .expect("Failed to serialize weapon message");

        effects.push(Effect::SetShootNow { slot: 0, value: 0 });
        effects.push(Effect::SendReliable(weapon_msg));
    }

    effects
}

pub fn game_wait_for_race(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    let mut effects = send_everything(ctr);

    // only send once and after camera fly-in is done
    if !state.race.flags.sent_start_race && (ctr.game_mode & 0x40) == 0 {
        let client_message = StartRace::new()
            .to_bytes()
            .expect("Failed to serialize start race message");

        effects.push(Effect::SendReliable(client_message));
        state.race.flags.sent_start_race = true;
    }

    effects
}

pub fn game_start_race(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    let mut effects = send_everything(ctr);

    // demo camera mode
    if ctr.gamemodes[Gamemode::DemoCamera as usize] {
        if ctr.cutscene_level_id < 18 {
            effects.push(Effect::WriteU16(0x80098028, 0x20));
        }
    }

    let warpclock = ctr.warpclock as i32;

    // stop orb/clock spam
    if !state.race.flags.sent_warpclock && state.race.warpclock_delay == 0.0 {
        let prev = state.previous.warpclock;
        if prev != Some(warpclock) {
            let client_message = WarpClock::new(warpclock as u8)
                .to_bytes()
                .expect("Failed to serialize warpclock message");

            effects.push(Effect::SendReliable(client_message));

            state.race.flags.sent_warpclock = true;
            state.previous.warpclock = Some(warpclock);
            state.race.warpclock_delay = ctr.now_secs;
        }
    }

    // set banned time for orb/clock
    if state.race.flags.sent_warpclock {
        state.race.timers[0] = ctr.now_secs - state.race.warpclock_delay;

        if state.race.timers[0] >= 50.0 {
            if ctr.warpclock != 0 {
                let client_message = WarpClock::new(0)
                    .to_bytes()
                    .expect("Failed to serialize warpclock message");

                effects.push(Effect::SendReliable(client_message));
            }

            state.race.flags.sent_warpclock = false;
            state.race.warpclock_delay = 0.0;
            state.race.timers[0] = 0.0;
        }
    }

    // calculate disconnected players
    let mut active = 0;
    for i in 0..ctr.driver_count as usize {
        if ctr.name_buffer[i][0] != 0 {
            active += 1;
        }
    }
    let required = if active >= 4 && state.race.extra_laps == 0 {
        3
    } else if active >= 4 && state.race.extra_laps != 0 {
        if active >= 5 { 4 } else { 3 }
    } else if active == 3 {
        2
    } else if active == 2 {
        1
    } else {
        0
    };
    let drivers_ended = ctr.drivers_ended_count as i32;
    let finish_race_timer = ctr.finish_race_timer;

    // if not 1 player race then set 30 seconds
    if drivers_ended == required && required != 0 && state.previous.finish_timer != Some(30) {
        let timer: u8 = if state.race.extra_laps != 0 { 60 } else { 30 };
        effects.push(Effect::SetFinishRaceTimer(timer));
        state.previous.finish_timer = Some(30);
    }

    // send the timer (visual) to the server
    if finish_race_timer > 0 && !state.race.flags.packet_already_sent {
        let client_message = FinishTimer::new(finish_race_timer)
            .to_bytes()
            .expect("Failed to serialize finish_timer message");

        effects.push(Effect::SendReliable(client_message));
        state.race.flags.packet_already_sent = true;
    }

    effects
}

pub fn game_end_race(ctr: &OnlineCtrSnapshot, state: &mut GameState) -> Vec<Effect> {
    let mut effects: Vec<Effect> = Vec::new();

    if !state.race.flags.sent_end_race {
        let course_time = ctr.race_course_time;
        let best_lap = ctr.race_best_lap;

        let client_message = EndRace::new(course_time, best_lap)
            .to_bytes()
            .expect("Failed to serialize end_race message");

        effects.push(Effect::SendReliable(client_message));

        let ended = ctr.drivers_ended_count as usize;
        effects.push(Effect::WriteRaceStats {
            slot: ended,
            stats: RaceStats {
                slot: 0,
                final_time: course_time,
                best_lap,
            },
        });
        effects.push(Effect::SetDriversEndedCount(ctr.drivers_ended_count + 1));

        state.race.flags.sent_end_race = true;
    }

    if state.race.flags.sent_end_race {
        let mut active = 0;
        for i in 0..ctr.driver_count as usize {
            if ctr.name_buffer[i][0] != 0 {
                active += 1;
            }
        }
        let ended = ctr.drivers_ended_count as i32;
        let finish_race_timer = ctr.finish_race_timer;

        if ended == active
            && state.previous.finish_timer != Some(3)
            && state.previous.finish_timer != Some(6)
        {
            let timer: u8 = if active == 1 { 6 } else { 3 };
            effects.push(Effect::SetFinishRaceTimer(timer));
            state.previous.finish_timer = Some(timer as i32);
            state.race.flags.packet_already_sent = true;
        }

        let needs_send = finish_race_timer > 0
            && state.race.flags.packet_already_sent
            && (state.previous.finish_timer == Some(3) || state.previous.finish_timer == Some(6));

        if needs_send {
            let client_message = FinishTimer::new(finish_race_timer)
                .to_bytes()
                .expect("Failed to serialize finish_timer message");

            effects.push(Effect::SendReliable(client_message));
            state.race.flags.packet_already_sent = false;
        }
    }

    effects
}

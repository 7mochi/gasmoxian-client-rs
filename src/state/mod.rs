use std::{
    net::SocketAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use deku::DekuContainerWrite;
use rusty_enet::Event::{self};

use crate::{
    console,
    enet::EnetClient,
    protocol::{
        ClientState,
        Gamemode::{self},
        MAX_NUM_PLAYERS,
        client::{
            Character, EndRace, Engine, FinishTimer, Kart, Password, Room, RoomType,
            RoomTypePassword, Special, StartRace, Track, WarpClock, Weapon,
        },
    },
    ps1_memory::{
        CHARACTER_ID, GAMEMODE, GAMEPAD_BASE, LOADING_STAGE, LOBBY_LEVEL_ID, PSX_POINTER, Ps1Memory,
    },
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
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };
    while let Ok(Some(event)) = net.poll() {
        match event {
            Event::Receive { packet, .. } => {
                handlers::process_receive_event(ps1_memory, net, state, packet.data());
            }
            Event::Disconnect { .. } => {
                console::err("Connection Dropped (Server Full or Server Offline)...");

                state.race.flags.password_sent = false;

                ps1_memory.online_ctr_mut().current_state = -1;
            }
            _ => {}
        }
    }
}

pub fn frame_stall(ps1_memory: &mut Ps1Memory) {
    while ps1_memory.online_ctr().ready_to_send == 0 {
        std::thread::sleep(Duration::from_micros(1));
    }

    ps1_memory.online_ctr_mut().ready_to_send = 0;
}

pub fn disconnect(ps1_memory: &mut Ps1Memory, net: Option<&mut EnetClient>, state: &mut GameState) {
    let hold = ps1_memory
        .read_u32(GAMEPAD_BASE + 0x10)
        .expect("GAMEPAD_BASE is within shared memory bounds");

    if (hold & 0x2000) != 0 {
        console::info("Disconnected from server (the player pressed DSELECT)");
        if let Some(net) = net {
            net.disconnect_now();
        }

        state.race.flags.lock_engine_and_character = false;

        ps1_memory.online_ctr_mut().auto_retry_join_room_index = -1;
        ps1_memory.online_ctr_mut().room_type = 0;
        ps1_memory.online_ctr_mut().room_type_locked = 0;
        ps1_memory.online_ctr_mut().current_state = -1;
    }
}

pub fn afk_timer(ps1_memory: &mut Ps1Memory, net: Option<&mut EnetClient>, state: &mut GameState) {
    if !state.race.flags.lock_engine_and_character {
        state.race.time_start = 0.0;
        return;
    }

    if state.race.time_start == 0.0 {
        state.race.time_start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before Unix epoch")
            .as_secs_f64();
        console::debug(format!("AFK timer started (timeout: {}s)", AFK_TIMEOUT));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before Unix epoch")
        .as_secs_f64();
    if (now - state.race.time_start) >= AFK_TIMEOUT {
        if !state.race.flags.lock_engine_and_character {
            state.race.time_start = 0.0;
            return;
        }
        console::err("Kicked for AFK");
        if let Some(net) = net {
            net.disconnect_now();
        }

        ps1_memory.online_ctr_mut().room_type = 0;
        ps1_memory.online_ctr_mut().room_type_locked = 0;
        ps1_memory.online_ctr_mut().current_state = -1;
        state.race.flags.lock_engine_and_character = false;
        state.race.time_start = 0.0;
    }
}

pub fn launch_enter_pid(ps1_memory: &mut Ps1Memory) {
    if ps1_memory.online_ctr().is_booted_ps1 == 0 {
        return;
    }
    console::debug(format!(
        "PS1 booted (psx_version: {}, pc_version: {})",
        ps1_memory.online_ctr().psx_version,
        ps1_memory.online_ctr().pc_version
    ));

    console::ok("Connected to DuckStation");
    console::info("Waiting to connect to a server...");

    ps1_memory.online_ctr_mut().current_state = ClientState::LaunchPickServer as i32;
}

pub fn launch_pick_server(ps1_memory: &mut Ps1Memory, state: &mut GameState) {
    // quit if disconnected but not loaded, back into the selection screen yet
    let level_id = ps1_memory
        .read_u32(GAMEMODE.wrapping_add(0x1a10))
        .expect("GAMEMODE is within shared memory bounds") as i8;

    // must be in cutscene level to see country selector
    if level_id != LOBBY_LEVEL_ID {
        return;
    }

    // quit if in loading screen (force-reconnect)
    let loading = ps1_memory
        .read_u32(LOADING_STAGE)
        .expect("LOADING_STAGE is within shared memory bounds");
    if loading != 0xFFFFFFFF {
        return;
    }

    let server_country = {
        let online_ctr = ps1_memory.online_ctr();
        // return now if the server selection hasn't been selected yet
        if online_ctr.server_lock_in1 == 0 {
            return;
        }

        online_ctr.server_country as usize
    };

    console::ok("Ready for server selection.");

    // TODO: private server

    // now selecting country
    ps1_memory.online_ctr_mut().client_busy = 1;
    let server = &SERVERS[server_country];

    if let Some(addr) = server.resolve() {
        state.connection.server_addr = Some(addr);
    } else {
        return;
    }
    state.connection.static_server_id = server_country as i32;

    console::info(format!("Ready to connect to {}", server.endpoint));
}

pub fn launch_error(_ps1_memory: &mut Ps1Memory) {
    // Version mismatch or other connection error — wait for user to return to menu.
}

pub fn launch_enter_password(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    if state.race.flags.password_sent {
        return;
    }

    let net = match net {
        Some(n) => n,
        None => return,
    };

    // keep alive via enet ping to prevent timeout
    state.race.count_frame += 1;
    if state.race.count_frame >= 60 {
        state.race.count_frame = 0;
        net.ping();
    }

    if ps1_memory.online_ctr().password_entered[7] == 0 {
        return;
    }

    let mut sequence = [0u8; 8];
    sequence.copy_from_slice(&ps1_memory.online_ctr().room_password_sequence);

    let client_message = Password::new(sequence)
        .to_bytes()
        .expect("Failed to serialize password message");

    net.send_reliable(&client_message)
        .expect("Failed to send password message");

    state.race.flags.password_sent = true;
}

pub fn lobby_assign_role(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    state.connection.attempt = 0;
    state.race.count_frame = 0;

    let net = match net {
        Some(n) => n,
        None => return,
    };

    // guest: do nothing
    if ps1_memory.online_ctr().driver_id > 0 {
        return;
    }

    if ps1_memory.online_ctr().room_type_locked == 0 {
        return;
    }

    let room_type = ps1_memory.online_ctr().room_type;
    let room_type_locked = ps1_memory.online_ctr().room_type_locked;

    if room_type == 2 {
        let mut sequence = [0u8; 8];
        sequence.copy_from_slice(&ps1_memory.online_ctr().room_password_sequence);

        let client_message = RoomTypePassword::new(room_type, room_type_locked, sequence)
            .to_bytes()
            .expect("Failed to serialize room type password message");

        net.send_reliable(&client_message)
            .expect("Failed to send room type password message");
    } else {
        let client_message = RoomType::new(room_type, room_type_locked)
            .to_bytes()
            .expect("Failed to serialize room type message");

        net.send_reliable(&client_message)
            .expect("Failed to send room type message");
    }
}

pub fn lobby_character_pick(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    let character_id = ps1_memory.read_u8(CHARACTER_ID).expect("read CHARACTER_ID") as i32;
    let locked_in = ps1_memory.online_ctr().locked_in_characters
        [ps1_memory.online_ctr().driver_id as usize] as i32;

    let previous_selection = &mut state.previous_selection;
    let has_changed = previous_selection.character_id != Some(character_id)
        || previous_selection.is_character_locked != (locked_in != 0);

    if has_changed {
        previous_selection.character_id = Some(character_id);
        previous_selection.is_character_locked = locked_in != 0;

        let client_message = Character::new(character_id as u8, locked_in != 0)
            .to_bytes()
            .expect("Failed to serialize character message");

        net.send_reliable(&client_message)
            .expect("Failed to send character message");
    }

    if locked_in != 0 {
        ps1_memory.online_ctr_mut().current_state = ClientState::LobbyEnginePick as i32;
    }
}

pub fn lobby_engine_pick(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    let engine_type = ps1_memory.online_ctr().engine_type[0] as i32;
    let locked_in = ps1_memory.online_ctr().locked_in_engines
        [ps1_memory.online_ctr().driver_id as usize] as i32;

    let previous_selection = &mut state.previous_selection;
    let has_changed = previous_selection.engine_type != Some(engine_type)
        || previous_selection.is_engine_locked != (locked_in != 0);

    if has_changed {
        previous_selection.engine_type = Some(engine_type);
        previous_selection.is_engine_locked = locked_in != 0;

        let client_message = Engine::new(engine_type as u8, locked_in != 0)
            .to_bytes()
            .expect("Failed to serialize engine message");

        net.send_reliable(&client_message)
            .expect("Failed to send engine message");
    }

    if locked_in != 0 {
        state.race.flags.lock_engine_and_character = false;

        ps1_memory.online_ctr_mut().current_state = ClientState::LobbyWaitForLoading as i32;
    }
}

pub fn lobby_special_pick(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    _state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    if ps1_memory.online_ctr().locked_in_special == 0 {
        return;
    }

    let mut gamemodes = [false; 18];
    gamemodes.copy_from_slice(&ps1_memory.online_ctr().gamemodes);

    // always ensure GameMode::Normal is enabled
    gamemodes[Gamemode::Normal as usize] = true;

    let client_message = Special::new(gamemodes)
        .to_bytes()
        .expect("Failed to serialize special message");

    net.send_reliable(&client_message)
        .expect("Failed to send special message");

    ps1_memory.online_ctr_mut().current_state = ClientState::LobbyCharacterPick as i32;
}

pub fn lobby_host_track_pick(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    _state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    let (lap_id, track_id) = {
        // locked_in_lap gets set after locked_in_level already sets
        if ps1_memory.online_ctr().locked_in_lap == 0 {
            return;
        }
        (
            ps1_memory.online_ctr().lap_id,
            ps1_memory.online_ctr().level_id,
        )
    };

    let num_laps = if (4..=15).contains(&lap_id) {
        let lap_values = [10, 15, 20, 25, 30, 35, 40, 50, 69, 80, 90, 127];
        lap_values[(lap_id - 4) as usize]
    } else {
        (lap_id * 2) + 1
    };
    ps1_memory
        .write_u8(0x80096b20 + 0x1d33, num_laps)
        .expect("write num_laps failed");

    let client_message = Track::new(track_id, lap_id)
        .to_bytes()
        .expect("Failed to serialize track message");

    net.send_reliable(&client_message)
        .expect("Failed to send track message");

    ps1_memory.online_ctr_mut().current_state = ClientState::LobbySpecialPick as i32;
}

pub fn lobby_guest_track_wait(state: &mut GameState) {
    state.previous_selection.character_id = None;
    state.previous_selection.is_character_locked = false;
    state.previous_selection.engine_type = None;
    state.previous_selection.is_engine_locked = false;
}

pub fn lobby_wait_for_loading() {
    // if recv message to start loading, change state to StartLoading, this check happens in ProcessNewMessages
}

pub fn lobby_start_loading(ps1_memory: &mut Ps1Memory, state: &mut GameState) {
    ps1_memory.online_ctr_mut().finish_race_timer = 0;

    state.race.flags.sent_start_race = false;
    state.race.flags.sent_end_race = false;
}

pub fn launch_pick_room(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    state.race.count_frame += 1;

    // room not updating bug still happens if the number is not 60, i didnt tried 30 anyways
    if state.race.count_frame == 60 {
        state.race.count_frame = 0;

        // send junk data, this triggers server response
        let client_message = Room::new(0xFF)
            .to_bytes()
            .expect("Failed to serialize join room message");

        net.send_reliable(&client_message)
            .expect("Failed to send join room message");
    }

    // wait for room to be chosen
    if ps1_memory.online_ctr().server_lock_in2 == 0 {
        state.connection.attempt = 0;
        return;
    }

    // dont send ClientMsg::JoinRoom twice
    if state.connection.attempt == 1 {
        return;
    }
    state.connection.attempt = 1;

    let room = ps1_memory.online_ctr().server_room;
    ps1_memory.online_ctr_mut().auto_retry_join_room_index = -1;

    let client_message = Room::new(room)
        .to_bytes()
        .expect("Failed to serialize join room message");

    net.send_reliable(&client_message)
        .expect("Failed to send join room message");
}

fn send_everything(ps1_memory: &mut Ps1Memory, net: &mut EnetClient) {
    // position
    let hold_raw = ps1_memory
        .read_u32(GAMEPAD_BASE + 0x10)
        .expect("GAMEPAD_BASE is within shared memory bounds");

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

    // position
    let psx_pointer = ps1_memory
        .read_u32(PSX_POINTER)
        .expect("PSX_POINTER is within shared memory bounds")
        & 0xFFFFFF;

    // lossless compression, bottom byte is never used,
    // cause psx renders with 3 bytes, and top byte
    // is never used due to world scale (just pure luck)
    let position_x = (ps1_memory
        .read_u32(psx_pointer + 0x2d4)
        .expect("position_x is within shared memory bounds")
        / 256) as i16;
    let position_y = (ps1_memory
        .read_u32(psx_pointer + 0x2d8)
        .expect("position_y is within shared memory bounds")
        / 256) as i16;
    let position_z = (ps1_memory
        .read_u32(psx_pointer + 0x2dc)
        .expect("position_z is within shared memory bounds")
        / 256) as i16;

    // direction faced
    let angle = ps1_memory
        .read_u16(psx_pointer + 0x39a)
        .expect("angle is within shared memory bounds")
        & 0xfff;

    let wumpa = ps1_memory
        .read_u8(psx_pointer + 0x30)
        .expect("wumpa is within shared memory bounds");
    let reserves = ps1_memory
        .read_u16(psx_pointer + 0x3e2)
        .expect("reserves is within shared memory bounds");

    let kart_msg = Kart::new(
        wumpa,
        reserves > 200,
        (angle & 0x1f) as u8,
        (angle >> 5) as u8,
        hold as u8,
        position_x,
        position_y,
        position_z,
    )
    .to_bytes()
    .expect("Failed to serialize kart message");

    let _ = net.send_unsequenced(&kart_msg);

    if ps1_memory.online_ctr().shoot[0].now != 0 {
        let weapon_msg = Weapon::new(
            ps1_memory.online_ctr().shoot[0].juiced != 0,
            ps1_memory.online_ctr().shoot[0].flags,
            ps1_memory.online_ctr().shoot[0].weapon,
        )
        .to_bytes()
        .expect("Failed to serialize weapon message");

        ps1_memory.online_ctr_mut().shoot[0].now = 0;
        net.send_reliable(&weapon_msg)
            .expect("Failed to send weapon message");
    }
}

pub fn game_wait_for_race(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    let game_mode = ps1_memory
        .read_u32(GAMEMODE)
        .expect("GAMEMODE is within shared memory bounds");

    // only send once and after camera fly-in is done
    if !state.race.flags.sent_start_race && (game_mode & 0x40) == 0 {
        let client_message = StartRace::new()
            .to_bytes()
            .expect("Failed to serialize start race message");

        net.send_reliable(&client_message)
            .expect("Failed to send start race message");

        state.race.flags.sent_start_race = true;
    }

    send_everything(ps1_memory, net);
}

pub fn game_start_race(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    send_everything(ps1_memory, net);

    // demo camera mode
    if ps1_memory.online_ctr().gamemodes[Gamemode::DemoCamera as usize] {
        let level_id = ps1_memory
            .read_u32(GAMEMODE.wrapping_add(0x1a10))
            .expect("Failed to read level_id") as i8;
        if level_id < 18 {
            ps1_memory
                .write_u16(0x80098028, 0x20)
                .expect("Failed to write demo camera mode");
        }
    }

    let warpclock = ps1_memory.online_ctr().warpclock as i32;

    // stop orb/clock spam
    if !state.race.flags.sent_warpclock && state.race.warpclock_delay == 0.0 {
        let prev = state.previous.warpclock;
        if prev != Some(warpclock) {
            let client_message = WarpClock::new(warpclock as u8)
                .to_bytes()
                .expect("Failed to serialize warpclock message");

            net.send_reliable(&client_message)
                .expect("Failed to send warpclock message");

            state.race.flags.sent_warpclock = true;
            state.previous.warpclock = Some(warpclock);
            state.race.warpclock_delay = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime before UNIX_EPOCH")
                .as_secs_f64();
        }
    }

    // set banned time for orb/clock
    if state.race.flags.sent_warpclock {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH")
            .as_secs_f64();
        state.race.timers[0] = now - state.race.warpclock_delay;

        if state.race.timers[0] >= 50.0 {
            if ps1_memory.online_ctr().warpclock != 0 {
                let client_message = WarpClock::new(0)
                    .to_bytes()
                    .expect("Failed to serialize warpclock message");

                net.send_reliable(&client_message)
                    .expect("Failed to send warpclock message");
            }

            state.race.flags.sent_warpclock = false;
            state.race.warpclock_delay = 0.0;
            state.race.timers[0] = 0.0;
        }
    }

    // calculate disconnected players
    let (drivers_ended, finish_race_timer, required) = {
        let mut active = 0;
        for i in 0..ps1_memory.online_ctr().driver_count as usize {
            if ps1_memory.online_ctr().name_buffer[i][0] != 0 {
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
        (
            ps1_memory.online_ctr().drivers_ended_count as i32,
            ps1_memory.online_ctr().finish_race_timer,
            required,
        )
    };

    // if not 1 player race then set 30 seconds
    if drivers_ended == required && required != 0 && state.previous.finish_timer != Some(30) {
        let timer: u8 = if state.race.extra_laps != 0 { 60 } else { 30 };
        ps1_memory.online_ctr_mut().finish_race_timer = timer;
        state.previous.finish_timer = Some(30);
    }

    // send the timer (visual) to the server
    if finish_race_timer > 0 && !state.race.flags.packet_already_sent {
        let client_message = FinishTimer::new(finish_race_timer)
            .to_bytes()
            .expect("Failed to serialize finish_timer message");

        net.send_reliable(&client_message)
            .expect("Failed to send finish_timer message");

        state.race.flags.packet_already_sent = true;
    }
}

pub fn game_end_race(
    ps1_memory: &mut Ps1Memory,
    net: Option<&mut EnetClient>,
    state: &mut GameState,
) {
    let net = match net {
        Some(n) => n,
        None => return,
    };

    if !state.race.flags.sent_end_race {
        let psx_ptr = ps1_memory
            .read_u32(PSX_POINTER)
            .expect("PSX_POINTER is within shared memory bounds")
            & 0xFFFFFF;

        let course_time = ps1_memory
            .read_u32(psx_ptr + 0x514)
            .expect("course_time is within shared memory bounds") as i32;
        let best_lap = ps1_memory
            .read_u32(psx_ptr + 0x63C)
            .expect("best_lap is within shared memory bounds") as i32;

        let client_message = EndRace::new(course_time, best_lap)
            .to_bytes()
            .expect("Failed to serialize end_race message");

        net.send_reliable(&client_message)
            .expect("Failed to send end_race message");

        let ended = ps1_memory.online_ctr().drivers_ended_count as usize;
        ps1_memory.online_ctr_mut().race_stats[ended].slot = 0;
        ps1_memory.online_ctr_mut().race_stats[ended].final_time = course_time;
        ps1_memory.online_ctr_mut().race_stats[ended].best_lap = best_lap;
        ps1_memory.online_ctr_mut().drivers_ended_count += 1;

        state.race.flags.sent_end_race = true;
    }

    if state.race.flags.sent_end_race {
        let mut active = 0;
        for i in 0..ps1_memory.online_ctr().driver_count as usize {
            if ps1_memory.online_ctr().name_buffer[i][0] != 0 {
                active += 1;
            }
        }
        let ended = ps1_memory.online_ctr().drivers_ended_count as i32;
        let finish_race_timer = ps1_memory.online_ctr().finish_race_timer;

        if ended == active
            && state.previous.finish_timer != Some(3)
            && state.previous.finish_timer != Some(6)
        {
            let timer: u8 = if active == 1 { 6 } else { 3 };
            ps1_memory.online_ctr_mut().finish_race_timer = timer;
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

            net.send_reliable(&client_message)
                .expect("Failed to send finish_timer message");

            state.race.flags.packet_already_sent = false;
        }
    }
}

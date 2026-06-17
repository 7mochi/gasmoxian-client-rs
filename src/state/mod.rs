use std::{
    net::SocketAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use deku::DekuContainerWrite;
use rusty_enet::Event::{self};

use crate::{
    console,
    enet::EnetClient,
    protocol::{ClientMessage, ClientState, MAX_NUM_PLAYERS, client::Room},
    ps1_memory::{GAMEMODE, GAMEPAD_BASE, LOADING_STAGE, LOBBY_LEVEL_ID, Ps1Memory},
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

    ps1_memory.online_ctr_mut().current_state = ClientState::LaunchPickRoom as i32;
}

pub fn launch_pick_room(ps1_memory: &mut Ps1Memory, net: Option<&mut EnetClient>, state: &mut GameState) {
    let net = match net {
        Some(n) => n,
        None => return,
    };
    
    state.race.count_frame += 1;

    // room not updating bug still happens if the number is not 60, i didnt tried 30 anyways
    if state.race.count_frame == 60 {
        state.race.count_frame = 0;

        // send junk data, this triggers server response
        let client_message = Room {
            msg_type: ClientMessage::JoinRoom,
            room: 0xFF,
        }
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

    let client_message = Room {
        msg_type: ClientMessage::JoinRoom,
        room,
    }
    .to_bytes()
    .expect("Failed to serialize join room message");

    net.send_reliable(&client_message)
        .expect("Failed to send join room message");
}

use std::fs;
use std::net::ToSocketAddrs;

use rusty_enet as enet;

use crate::{
    console,
    enet::EnetClient,
    protocol::{
        ClientState, DRIVER_BESTLAP_OFFSET, DRIVER_COURSE_OFFSET, GASMOXIAN_VERSION, GameMode,
        LOBBY_LEVEL_ID, MAX_NUM_PLAYERS, NAME_LENGTH, ServerMsg, client, server,
    },
    ps1_memory::{
        CHARACTER_ID, CHEATS, GAME_MODE, GAMEPAD_BASE, LOADING_STAGE, PSX_POINTER, Ps1Memory,
    },
    servers::SERVERS,
};

/// Global mutable state for the Gasmoxian game client.
///
/// Mirrors the scattered global variables in the original C++ codebase.
/// All 16 state functions receive this via `&mut GameState` and modify it in place.
pub struct GameState {
    pub connection_attempt: i32,
    pub count_frame: i32,

    pub password_sent: bool,

    pub lock_engine_and_character: bool,

    pub previous_character_id: i32,
    pub previous_bool_locked_in: i32,
    pub previous_enginetype: i32,
    pub previous_bool_locked_in_engine: i32,

    pub already_sended: i32,
    pub send_warpclock: i32,
    pub extra_laps: i32,

    pub already_sent_start_race: i32,
    pub already_sent_end_race: i32,

    pub previous_warpclock: i32,
    pub previous_special: i32,
    pub previous_finish_timer: i32,

    pub time_start: f64,
    pub warpclock_delay: f64,
    pub square_delay: [u64; MAX_NUM_PLAYERS],

    pub timers: [f64; 2],

    pub previous_button: [i32; 8],

    pub name: [u8; NAME_LENGTH + 1],

    pub static_server_id: i32,
    pub static_room_id: i32,

    pub server_addr: Option<std::net::SocketAddr>,

    pub required_players: i32,
    pub disconnected_players: i32,
    pub active_players: i32,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            connection_attempt: 0,
            count_frame: 0,
            password_sent: false,
            lock_engine_and_character: false,
            previous_character_id: -1,
            previous_bool_locked_in: -1,
            previous_enginetype: -1,
            previous_bool_locked_in_engine: -1,
            already_sended: 0,
            send_warpclock: 0,
            extra_laps: 0,
            already_sent_start_race: 0,
            already_sent_end_race: 0,
            previous_warpclock: -1,
            previous_special: -1,
            previous_finish_timer: -1,
            time_start: 0.0,
            warpclock_delay: 0.0,
            square_delay: [0; MAX_NUM_PLAYERS],
            timers: [0.0, 0.0],
            previous_button: [0; 8],
            name: [0; NAME_LENGTH + 1],
            static_server_id: 0,
            static_room_id: 0,
            server_addr: None,
            required_players: 0,
            disconnected_players: 0,
            active_players: 0,
        }
    }
}

fn resolve_server_address(ip: &str, port: u16) -> Option<std::net::SocketAddr> {
    let addr_str = format!("{}:{}", ip, port);
    match addr_str.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => Some(a),
            None => {
                console::err(format!("Could not resolve server: {}", ip));
                None
            }
        },
        Err(e) => {
            console::err(format!("Failed to resolve {}: {}", ip, e));
            None
        }
    }
}

/// Signature for a single client state function.
///
/// Each function receives the PS1 shared memory, the ENet client connection,
/// and the mutable game state
pub type StateFn = fn(&Ps1Memory, &mut EnetClient, &mut GameState);

/// Array of 16 state function pointers indexed by [`ClientState`].
///
/// Dispatch is done by reading `ps1.octr().current_state` and calling
/// `STATE_FUNCTIONS[idx]()`. Mirrors the C++ `ClientState[]` array.
pub const STATE_FUNCTIONS: [StateFn; 16] = [
    launch_enter_pid,       // 0
    launch_pick_server,     // 1
    launch_pick_room,       // 2
    launch_error,           // 3
    launch_enter_password,  // 4
    lobby_assign_role,      // 5
    lobby_host_track_pick,  // 6
    lobby_special_pick,     // 7
    lobby_guest_track_wait, // 8
    lobby_character_pick,   // 9
    lobby_engine_pick,      // 10
    lobby_wait_for_loading, // 11
    lobby_start_loading,    // 12
    game_wait_for_race,     // 13
    game_start_race,        // 14
    game_end_race,          // 15
];

fn launch_enter_pid(ps1: &Ps1Memory, _net: &mut EnetClient, _state: &mut GameState) {
    if ps1.online_ctr().is_booted_ps1 == 0 {
        return;
    }

    console::spinner_ok("Connected to DuckStation");
    console::info("Waiting to connect to a server...");

    ps1.online_ctr_mut().current_state = ClientState::LaunchPickServer as i32;
}

fn launch_pick_server(ps1: &Ps1Memory, _net: &mut EnetClient, state: &mut GameState) {
    // quit if disconnected, but not loaded
	// back into the selection screen yet
    let level_id = ps1.read_u32(GAME_MODE.wrapping_add(0x1a10)) as i32;
    
    // must be in cutscene level to see country selector
    if level_id != LOBBY_LEVEL_ID {
        return;
    }

    // quit if in loading screen (force-reconnect)
    let loading = ps1.read_u32(LOADING_STAGE);
    if loading != 0xFFFFFFFF {
        return;
    }

    let server_country = {
        let octr = ps1.online_ctr();
        // return now if the server selection hasn't been selected yet
        if octr.server_lock_in1 == 0 {
            return;
        }
        octr.server_country as usize
    };

    console::spinner_ok("Ready for server selection");

    // private server
    if server_country >= SERVERS.len() {
        let private = match fs::read_to_string("data/host/host.txt") {
            Ok(content) => {
                let mut lines = content.lines();
                let ip = lines.next().unwrap_or("").trim().to_string();
                let port_str = lines.next().unwrap_or("0").trim();
                let port: u16 = match port_str.parse() {
                    Ok(p) if p > 0 => p,
                    _ => {
                        console::err("Invalid port in data/host/host.txt");
                        return;
                    }
                };
                Some((ip, port))
            }
            Err(_) => console::prompt_private_server(),
        };

        let (ip, port) = match private {
            Some(p) => p,
            None => return,
        };

        state.static_server_id = SERVERS.len() as i32; // out of bounds index to indicate private server

        let addr = match resolve_server_address(&ip, port) {
            Some(a) => a,
            None => return,
        };

        state.server_addr = Some(addr);

        console::info(format!("Ready to connect to private server \"{}\"", ip));
        ps1.online_ctr_mut().current_state = ClientState::LaunchPickRoom as i32;
        
        return;
    }

    // now selecting country
    ps1.online_ctr_mut().client_busy = 1;
    let server = &SERVERS[server_country];
    
    let addr = match resolve_server_address(server.address, server.port) {
        Some(a) => a,
        None => return,
    };

    state.static_server_id = server_country as i32;
    state.server_addr = Some(addr);

    console::info(format!("Ready to connect to \"{}\"", server.address));
    ps1.online_ctr_mut().current_state = ClientState::LaunchPickRoom as i32;
}

fn launch_pick_room(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    state.count_frame += 1;
    
    // room not updating bug still happens if the number is not 60, i didnt tried 30 anyways
    if state.count_frame == 60 {
        state.count_frame = 0;

        // send junk data, this triggers server response
        net.send_reliable(&client::MessageRoom { room: 0xFF }.to_bytes());
    }

    // wait for room to be chosen
    if ps1.online_ctr().server_lock_in2 == 0 {
        state.connection_attempt = 0;
        return;
    }

    // dont send ClientMsg::JoinRoom twice
    if state.connection_attempt == 1 {
        return;
    }
    state.connection_attempt = 1;

    let room = ps1.online_ctr().server_room;
    ps1.online_ctr_mut().auto_retry_join_room_index = -1;

    net.send_reliable(&client::MessageRoom { room }.to_bytes());
}

fn launch_error(_ps1: &Ps1Memory, _net: &mut EnetClient, _state: &mut GameState) {
    // do nothing
}

fn launch_enter_password(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    if state.password_sent {
        return;
    }

    // keep alive via Enet ping to prevent timeout
    state.count_frame += 1;
    if state.count_frame >= 60 {
        state.count_frame = 0;
        net.ping();
    }

    if ps1.online_ctr().password_entered[7] == 0 {
        return;
    }

    let mut seq = [0u8; 8];
    for i in 0..8 {
        seq[i] = ps1.online_ctr().room_password_sequence[i];
    }
    net.send_reliable(&client::MessagePassword { sequence: seq }.to_bytes());

    state.password_sent = true;
}

fn lobby_assign_role(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    state.connection_attempt = 0;
    state.count_frame = 0;

    if ps1.online_ctr().driver_id > 0 {
        return; // guest: do nothing
    }

    if ps1.online_ctr().room_type_locked == 0 {
        return;
    }

    let room_type = ps1.online_ctr().room_type;
    let room_type_locked = ps1.online_ctr().room_type_locked;
    let room_type_name = match room_type {
        1 => "TOURNAMENT",
        2 => "PASSWORD",
        _ => "NORMAL",
    };
    console::info(format!("Sending room type ({})...", room_type_name));

    if room_type == 2 {
        let mut seq = [0u8; 8];
        for i in 0..8 {
            seq[i] = ps1.online_ctr().room_password_sequence[i];
        }
        net.send_reliable(
            &client::MessageRoomTypePassword {
                room_type,
                r_type_locked: room_type_locked,
                sequence: seq,
            }
            .to_bytes(),
        );
    } else {
        net.send_reliable(
            &client::MessageRoomType {
                room_type,
                r_type_locked: room_type_locked,
            }
            .to_bytes(),
        );
    }
}

fn lobby_host_track_pick(ps1: &Ps1Memory, net: &mut EnetClient, _state: &mut GameState) {
    let (lap_id, track_id) = {
        let octr = ps1.online_ctr();
        
        // locked_in_lap gets set after locked_in_level already sets
        if octr.locked_in_lap == 0 {
            return;
        }
        (octr.lap_id, octr.level_id)
    };

    let num_laps = if lap_id >= 4 && lap_id <= 15 {
        let lap_values = [10, 15, 20, 25, 30, 35, 40, 50, 69, 80, 90, 127];
        lap_values[(lap_id - 4) as usize]
    } else {
        (lap_id * 2) + 1
    };
    ps1.write_u8(0x80096b20 + 0x1d33, num_laps);

    console::spinner_ok("Track sent");
    console::info("Sending track to the server...");

    net.send_reliable(&client::MessageTrack { track_id, lap_id }.to_bytes());

    ps1.online_ctr_mut().current_state = ClientState::LobbySpecialPick as i32;
}

fn lobby_special_pick(ps1: &Ps1Memory, net: &mut EnetClient, _state: &mut GameState) {
    let mut gamemodes = [false; 18];
    {
        let octr = ps1.online_ctr();
        if octr.locked_in_special == 0 {
            return;
        }
        for i in 0..18 {
            gamemodes[i] = octr.gamemodes[i];
        }
        
        // always ensure GameMode::Normal is enabled
        gamemodes[GameMode::Normal as usize] = true;
    }

    console::spinner_ok("Gamemodes sent");
    console::info("Sending gamemodes to the server...");

    // send the entire structure to the server
    net.send_reliable(&client::MessageSpecial { gamemodes }.to_bytes());

    ps1.online_ctr_mut().current_state = ClientState::LobbyCharacterPick as i32;
}

fn lobby_guest_track_wait(_ps1: &Ps1Memory, _net: &mut EnetClient, state: &mut GameState) {
    state.previous_character_id = -1;
    state.previous_bool_locked_in = -1;
    state.previous_enginetype = -1;
    state.previous_bool_locked_in_engine = -1;
}

fn lobby_character_pick(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    let character_id = ps1.read_u8(CHARACTER_ID) as i32;
    let bool_locked_in =
        ps1.online_ctr().locked_in_characters[ps1.online_ctr().driver_id as usize] as i32;

    if state.previous_character_id != character_id
        || state.previous_bool_locked_in != bool_locked_in
    {
        state.previous_character_id = character_id;
        state.previous_bool_locked_in = bool_locked_in;
        net.send_reliable(
            &client::MessageCharacter {
                character_id: character_id as u8,
                locked_in: bool_locked_in != 0,
            }
            .to_bytes(),
        );
    }

    if bool_locked_in != 0 {
        ps1.online_ctr_mut().current_state = ClientState::LobbyEnginePick as i32;
    }
}

fn lobby_engine_pick(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    let engine_type = ps1.online_ctr().engine_type[0] as i32; // slot is not the same than octr().driver_id.
    let bool_locked_in =
        ps1.online_ctr().locked_in_engines[ps1.online_ctr().driver_id as usize] as i32;
    
    if state.previous_enginetype != engine_type
        || state.previous_bool_locked_in_engine != bool_locked_in
    {
        state.previous_enginetype = engine_type;
        state.previous_bool_locked_in_engine = bool_locked_in;
        net.send_reliable(
            &client::MessageEngine {
                engine_type: engine_type as u8,
                locked_in: bool_locked_in != 0,
            }
            .to_bytes(),
        );
    }

    if bool_locked_in != 0 {
        state.lock_engine_and_character = false;
        ps1.online_ctr_mut().current_state = ClientState::LobbyWaitForLoading as i32;
    }
}

fn lobby_wait_for_loading(_ps1: &Ps1Memory, _net: &mut EnetClient, _state: &mut GameState) {
    // if recv message to start loading, change state to StartLoading, this check happens in ProcessNewMessages
}

fn lobby_start_loading(ps1: &Ps1Memory, _net: &mut EnetClient, state: &mut GameState) {
    ps1.online_ctr_mut().finish_race_timer = 0;
    state.already_sent_start_race = 0;
    state.already_sent_end_race = 0;
}

fn game_wait_for_race(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    let game_mode = ps1.read_u32(GAME_MODE);

    // only send once and after camera fly-in is done
    if state.already_sent_start_race == 0 && (game_mode & 0x40) == 0 {
        console::spinner_ok("Race started");
        console::info("Gasmoxian race in progress...");
        state.already_sent_start_race = 1;
        net.send_reliable(&client::Header::to_bytes());
    }
    send_everything(ps1, net, state);
}

fn game_start_race(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    send_everything(ps1, net, state);

    // demo camera mode
    if ps1.online_ctr().gamemodes[GameMode::DemoCamera as usize] {
        let level_id = ps1.read_u32(GAME_MODE.wrapping_add(0x1a10));
        if level_id < 18 {
            ps1.write_u16(0x80098028, 0x20);
        }
    }

    let warpclock = ps1.online_ctr().warpclock as i32;

    // stop orb/clock spam
    if state.send_warpclock == 0 && state.warpclock_delay == 0.0 {
        if warpclock != state.previous_warpclock {
            net.send_reliable(
                &client::MessageWarpclock {
                    warp_clock: warpclock as u8,
                }
                .to_bytes(),
            );
            state.send_warpclock = 1;
            state.previous_warpclock = warpclock;
            state.warpclock_delay = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("SystemTime before UNIX_EPOCH")
                .as_secs_f64();
        }
    }

    // set banned time for orb/clock
    if state.send_warpclock != 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        state.timers[0] = now - state.warpclock_delay;
        if state.timers[0] >= 50.0 {
            if ps1.online_ctr().warpclock != 0 {
                net.send_reliable(&client::MessageWarpclock { warp_clock: 0 }.to_bytes());
            }
            state.send_warpclock = 0;
            state.warpclock_delay = 0.0;
            state.timers[0] = 0.0;
        }
    }

    // calculate disconnected players
    let (_, finish_race_timer) = {
        let octr = ps1.online_ctr();
        let mut active = 0;
        for i in 0..octr.driver_count as usize {
            if octr.name_buffer[i][0] != 0 {
                active += 1;
            }
        }
        let required = if active >= 4 && state.extra_laps == 0 {
            3
        } else if active >= 4 && state.extra_laps != 0 {
            if active >= 5 { 4 } else { 3 }
        } else if active == 3 {
            2
        } else if active == 2 {
            1
        } else {
            0
        };

        // if not 1 player race then set 30 seconds
        if octr.drivers_ended_count as i32 == required
            && required != 0
            && state.previous_finish_timer != 30
        {
            let timer: u8 = if state.extra_laps != 0 { 60 } else { 30 };
            ps1.online_ctr_mut().finish_race_timer = timer;
            state.previous_finish_timer = 30;
        }

        (octr.drivers_ended_count, octr.finish_race_timer)
    };

    // send the timer (visual) to the server
    if finish_race_timer > 0 && state.already_sended == 0 {
        net.send_reliable(
            &client::MessageFinishTimer {
                finish_timer: finish_race_timer,
            }
            .to_bytes(),
        );
        state.already_sended = 1;
    }
}

fn game_end_race(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    if state.already_sent_end_race == 0 {
        state.already_sent_end_race = 1;

        let psx_ptr = (ps1.read_u32(PSX_POINTER) & 0xFFFFFF) as u32;
        let course_time = ps1.read_u32(psx_ptr + DRIVER_COURSE_OFFSET);
        let best_lap = ps1.read_u32(psx_ptr + DRIVER_BESTLAP_OFFSET);

        net.send_reliable(
            &client::MessageEndRace {
                course_time: course_time as i32,
                lap_time: best_lap as i32,
            }
            .to_bytes(),
        );

        let ended = ps1.online_ctr().drivers_ended_count as usize;
        ps1.online_ctr_mut().race_stats[ended].slot = 0;
        ps1.online_ctr_mut().race_stats[ended].final_time = course_time as i32;
        ps1.online_ctr_mut().race_stats[ended].best_lap = best_lap as i32;
        ps1.online_ctr_mut().drivers_ended_count += 1;
    }

    if state.already_sent_end_race != 0 {
        // GASMOX_CLIENT.cpp:1654-1674
        let (finish_race_timer, needs_send) = {
            let octr = ps1.online_ctr();
            let mut active = 0;
            for i in 0..octr.driver_count as usize {
                if octr.name_buffer[i][0] != 0 {
                    active += 1;
                }
            }
            let ended = octr.drivers_ended_count as i32;
            if ended == active
                && state.previous_finish_timer != 3
                && state.previous_finish_timer != 6
            {
                let timer: u8 = if active == 1 { 6 } else { 3 };
                ps1.online_ctr_mut().finish_race_timer = timer;
                state.previous_finish_timer = timer as i32;
                state.already_sended = 1;
            }

            let needs = octr.finish_race_timer > 0
                && state.already_sended != 0
                && (state.previous_finish_timer == 3 || state.previous_finish_timer == 6);
            (octr.finish_race_timer, needs)
        };

        if needs_send {
            net.send_reliable(
                &client::MessageFinishTimer {
                    finish_timer: finish_race_timer,
                }
                .to_bytes(),
            );
            state.already_sended = 0;
        }
    }
}

fn send_everything(ps1: &Ps1Memory, net: &mut EnetClient, _state: &mut GameState) {
    // position
    let hold_raw = ps1.read_u32(GAMEPAD_BASE + 0x10);

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
    let psx_pointer = (ps1.read_u32(PSX_POINTER) & 0xFFFFFF) as u32;

    // lossless compression, bottom byte is never used,
    // cause psx renders with 3 bytes, and top byte
    // is never used due to world scale (just pure luck)
    let position_x = (ps1.read_u32(psx_pointer + 0x2d4) / 256) as i16;
    let position_y = (ps1.read_u32(psx_pointer + 0x2d8) / 256) as i16;
    let position_z = (ps1.read_u32(psx_pointer + 0x2dc) / 256) as i16;

    // direction faced
    let angle = ps1.read_u16(psx_pointer + 0x39a) & 0xfff;

    let wumpa = ps1.read_u8(psx_pointer + 0x30);
    let reserves = ps1.read_u16(psx_pointer + 0x3e2);

    let kart = client::EverythingKart {
        wumpa,
        reserves: reserves > 200,
        kart_rotation1: (angle & 0x1f) as u8, // angle_bit_5
        kart_rotation2: (angle >> 5) as u8,   // angle_top_8
        button_hold: hold as u8,
        position_x,
        position_y,
        position_z,
    };

    net.send_unsequenced(&kart.to_bytes());

    let weapon_data = {
        let octr = ps1.online_ctr();
        if octr.shoot[0].now == 0 {
            None
        } else {
            Some((
                octr.shoot[0].weapon,
                octr.shoot[0].juiced != 0,
                octr.shoot[0].flags,
            ))
        }
    };

    if let Some((weapon, juiced, flags)) = weapon_data {
        ps1.online_ctr_mut().shoot[0].now = 0;
        let w = client::MessageWeapon {
            juiced,
            flags,
            weapon,
        };
        net.send_reliable(&w.to_bytes());
    }
}

/// Spins until the PS1 sets `ready_to_send`, then clears it.
///
/// Mirrors `FrameStall()` in the C++ client. Used to synchronize the client
/// loop with the emulated PS1 frame timing.
pub fn frame_stall(ps1: &Ps1Memory) {
    while ps1.online_ctr().ready_to_send == 0 {
        std::thread::sleep(std::time::Duration::from_micros(1));
    }
    ps1.online_ctr_mut().ready_to_send = 0;
}

/// Checks if the SELECT button is held and disconnects from the server.
///
/// Mirrors `DisconSELECT()` in the C++ client. Called every frame when
/// `curr_state >= LaunchPickRoom`.
pub fn discon_select(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    let hold = ps1.read_u32(GAMEPAD_BASE + 0x10);
    if (hold & 0x2000) != 0 {
        console::spinner_ok("Disconnected");
        console::info("Disconnected (ID: DSELECT)...");
        net.disconnect_now();
        state.lock_engine_and_character = false;
        ps1.online_ctr_mut().auto_retry_join_room_index = -1;
        ps1.online_ctr_mut().room_type = 0;
        ps1.online_ctr_mut().room_type_locked = 0;
        ps1.online_ctr_mut().current_state = -1;
    }
}

/// Anti-AFK timer – disconnects the player after 80 seconds of inactivity
/// during character/engine selection.
///
/// Mirrors `afktimer()` in the C++ client.
pub fn afk_timer(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    if !state.lock_engine_and_character {
        state.time_start = 0.0;
        return;
    }

    if state.time_start == 0.0 {
        state.time_start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    if (now - state.time_start) >= 80.0 {
        if !state.lock_engine_and_character {
            state.time_start = 0.0;
            return;
        }
        console::spinner_err("Kicked for AFK");
        net.disconnect_now();
        state.lock_engine_and_character = false;
        ps1.online_ctr_mut().room_type = 0;
        ps1.online_ctr_mut().room_type_locked = 0;
        ps1.online_ctr_mut().current_state = -1;
        state.time_start = 0.0;
    }
}

/// Polls the ENet connection for incoming packets and dispatches them.
///
/// Mirrors `ProcessNewMessages()` in the C++ client. Handles `Receive` events
/// by delegating to `process_receive_event()` and `Disconnect` events by
/// resetting the client state.
pub fn process_new_messages(ps1: &Ps1Memory, net: &mut EnetClient, state: &mut GameState) {
    while let Ok(Some(event)) = net.poll() {
        match event {
            enet::Event::Receive { packet, .. } => {
                process_receive_event(ps1, net, state, packet.data());
            }
            enet::Event::Disconnect { .. } => {
                console::err("Connection Dropped (Server Full or Server Offline)...");
                state.password_sent = false;
                ps1.online_ctr_mut().current_state = -1;
            }
            _ => {}
        }
    }
}

// GASMOX_CLIENT.cpp:152-665
fn process_receive_event(
    ps1: &Ps1Memory,
    net: &mut EnetClient,
    state: &mut GameState,
    data: &[u8],
) {
    let msg_type = data[0] & 0x0F;
    match msg_type {
        t if t == ServerMsg::Rooms as u8 => {
            // GASMOX_CLIENT.cpp:161-207
            let r = server::MessageRooms::from_bytes(data);

            ps1.online_ctr_mut().pc_version = GASMOXIAN_VERSION;
            ps1.online_ctr_mut().server_version = r.version as i32;

            if r.version != GASMOXIAN_VERSION as u16 {
                console::err(format!(
                    "Version mismatch! Server={}, Client={}",
                    r.version, GASMOXIAN_VERSION
                ));
                ps1.online_ctr_mut().current_state = ClientState::LaunchError as i32;
                return;
            }

            if ps1.online_ctr().psx_version != GASMOXIAN_VERSION {
                console::err("PSX version mismatch!");
                ps1.online_ctr_mut().current_state = ClientState::LaunchError as i32;
                return;
            }

            let curr_state = ps1.online_ctr().current_state;
            if curr_state == ClientState::LaunchEnterPassword as i32
                || curr_state >= ClientState::LobbyAssignRole as i32
            {
                return;
            }

            ps1.online_ctr_mut().server_lock_in2 = 0;
            ps1.online_ctr_mut().room_count = r.room_count;
            for i in 0..16 {
                ps1.online_ctr_mut().client_count[i] = r.client_count[i];
            }
        }
        t if t == ServerMsg::RoomType as u8 => {
            // GASMOX_CLIENT.cpp:209-224
            let r = server::MessageRoomType::from_bytes(data);
            ps1.online_ctr_mut().room_type = r.room_type;
            if r.room_type == 2
                && ps1.online_ctr().room_type_locked == 0
                && ps1.online_ctr().current_state == ClientState::LaunchPickRoom as i32
            {
                state.password_sent = false;
                ps1.online_ctr_mut().current_state = ClientState::LaunchEnterPassword as i32;
            }
        }
        t if t == ServerMsg::PasswordRejected as u8 => {
            // GASMOX_CLIENT.cpp:226-242
            console::spinner_err("Wrong password");
            console::err("Wrong password. Returning to room list.");
            net.disconnect_now();
            ps1.online_ctr_mut().room_type = 0;
            ps1.online_ctr_mut().room_type_locked = 0;
            state.lock_engine_and_character = false;
            ps1.online_ctr_mut().auto_retry_join_room_index = -1;
            for i in 0..8 {
                ps1.online_ctr_mut().password_entered[i] = 0;
                ps1.online_ctr_mut().room_password_sequence[i] = 0;
            }
            state.password_sent = false;
            ps1.online_ctr_mut().current_state = ClientState::LaunchPickRoom as i32;
        }
        t if t == ServerMsg::NewClient as u8 => {
            // GASMOX_CLIENT.cpp:244-300
            let r = server::MessageClientStatus::from_bytes(data);
            state.password_sent = false;
            ps1.online_ctr_mut().driver_id = r.client_id;
            ps1.online_ctr_mut().driver_count = r.client_count;
            ps1.online_ctr_mut().locked_in_lap = 0;
            ps1.online_ctr_mut().locked_in_level = 0;
            ps1.online_ctr_mut().locked_in_engine = 0;
            ps1.online_ctr_mut().locked_in_special = 0;
            ps1.online_ctr_mut().lap_id = 0;
            ps1.online_ctr_mut().special = 0;
            ps1.online_ctr_mut().level_id = 0;
            ps1.online_ctr_mut().locked_in_character = 0;
            ps1.online_ctr_mut().drivers_ended_count = 0;
            ps1.online_ctr_mut().finish_race_timer = 0;
            ps1.online_ctr_mut().warpclock = 0;
            state.lock_engine_and_character = false;
            state.already_sended = 0;
            state.send_warpclock = 0;
            state.previous_warpclock = -1;
            state.previous_special = -1;
            state.previous_finish_timer = -1;
            state.extra_laps = 0;

            if ps1.online_ctr().server_room == 15 {
                console::ok("EASTER EGG: SAFFI FIRE UNLOCKED IN THIS ROOM");
            }

            // Zero out all arrays
            for i in 0..MAX_NUM_PLAYERS {
                ps1.online_ctr_mut().locked_in_characters[i] = 0;
                ps1.online_ctr_mut().locked_in_engines[i] = 0;
                for j in 0..=NAME_LENGTH {
                    ps1.online_ctr_mut().name_buffer[i][j] = 0;
                }
                ps1.online_ctr_mut().race_stats[i].slot = 0;
                ps1.online_ctr_mut().race_stats[i].final_time = 0;
                ps1.online_ctr_mut().race_stats[i].best_lap = 0;
            }
            state.square_delay = [0; MAX_NUM_PLAYERS];
            for i in 0..8 {
                ps1.online_ctr_mut().password_entered[i] = 0;
                ps1.online_ctr_mut().room_password_sequence[i] = 0;
            }

            // Send name to server
            let name_buf = state.name;
            for i in 0..NAME_LENGTH {
                ps1.online_ctr_mut().name_buffer[0][i] = name_buf[i];
            }
            ps1.online_ctr_mut().name_buffer[0][NAME_LENGTH] = 0;

            let mut name = [0u8; 12];
            for i in 0..NAME_LENGTH {
                name[i] = name_buf[i];
            }
            net.send_reliable(&client::MessageName { name }.to_bytes());

            ps1.online_ctr_mut().current_state = ClientState::LobbyAssignRole as i32;
        }
        t if t == ServerMsg::Name as u8 => {
            // GASMOX_CLIENT.cpp:302-329
            let r = server::MessageName::from_bytes(data);
            let driver_id = ps1.online_ctr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                } as usize;
                ps1.online_ctr_mut().driver_count = r.client_count;
                for i in 0..NAME_LENGTH {
                    ps1.online_ctr_mut().name_buffer[slot][i] = r.name[i];
                }
                ps1.online_ctr_mut().name_buffer[slot][NAME_LENGTH] = 0;

                // Handle disconnection — force SQUARE if name starts with 0
                if r.name[0] == 0
                    || ps1.online_ctr().current_state <= ClientState::LobbyWaitForLoading as i32
                {
                    let gp_addr = GAMEPAD_BASE + (slot as u32 * 0x50);
                    ps1.write_u32(gp_addr, 0x20);
                    ps1.write_u32(gp_addr + 0x4, 0);
                    ps1.write_u32(gp_addr + 0x8, 0);
                    ps1.write_u32(gp_addr + 0xC, 0x20);
                }
            }
        }
        t if t == ServerMsg::Track as u8 => {
            // GASMOX_CLIENT.cpp:331-361
            let r = server::MessageTrack::from_bytes(data);
            let num_laps = if r.lap_id >= 4 && r.lap_id <= 15 {
                let lap_values = [10, 15, 20, 25, 30, 35, 40, 50, 69, 80, 90, 127];
                lap_values[(r.lap_id - 4) as usize]
            } else {
                (r.lap_id * 2) + 1
            };
            ps1.write_u8(0x80096b20 + 0x1d33, num_laps);
            state.extra_laps = if r.lap_id >= 8 { 1 } else { 0 };
            ps1.online_ctr_mut().level_id = r.track_id;
            ps1.online_ctr_mut().current_state = ClientState::LobbySpecialPick as i32;
        }
        t if t == ServerMsg::Special as u8 => {
            // GASMOX_CLIENT.cpp:363-410
            let r = server::MessageSpecial::from_bytes(data);
            let octr = ps1.online_ctr_mut();
            // Copy all gamemode toggles
            for i in 0..18 {
                octr.gamemodes[i] = r.gamemodes[i];
            }
            octr.gamemodes[GameMode::Normal as usize] = true;

            // Apply cheat effects
            let mut cheats = ps1.read_u32(CHEATS);
            cheats &= !(0x100000 | 0x80000 | 0x400 | 0x400000 | 0x8000000 | 0x10000);
            if octr.gamemodes[GameMode::IcyTracks as usize] {
                cheats |= 0x80000;
                console::ok("MODE: ICY TRACKS");
            }
            if octr.gamemodes[GameMode::RetroFueled as usize] {
                cheats |= 0x100000;
                console::ok("MODE: RETRO FUELED");
            }
            ps1.write_u32(CHEATS, cheats);

            for i in 0..18 {
                if i != GameMode::IcyTracks as usize && i != GameMode::RetroFueled as usize {
                    if octr.gamemodes[i] {
                        console::ok(format!(
                            "MODE: {} ENABLED",
                            match i {
                                0 => "NORMAL",
                                1 => "MIRROR",
                                2 => "ICY TRACKS",
                                3 => "ITEMLESS",
                                4 => "MOON MODE",
                                5 => "RETRO FUELED",
                                6 => "FIRST PERSON",
                                7 => "BOSS RACE",
                                8 => "DEMO CAMERA",
                                9 => "N. VERTED",
                                10 => "SHORTCUTLESS",
                                11 => "NIGHT",
                                12 => "DARKNESS",
                                13 => "ITEM CHAOS",
                                14 => "SURVIVAL",
                                15 => "SURVIVAL TIMER",
                                16 => "VANILLA ITEMS",
                                17 => "WALL DRIVE",
                                _ => "UNKNOWN",
                            }
                        ));
                    }
                }
            }
            ps1.online_ctr_mut().current_state = ClientState::LobbyCharacterPick as i32;
        }
        t if t == ServerMsg::Character as u8 => {
            // GASMOX_CLIENT.cpp:411-440
            let r = server::MessageCharacter::from_bytes(data);
            let driver_id = ps1.online_ctr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                };
                ps1.write_u16(0x80086e84 + (slot as u32 * 2), r.character_id as u16);
                ps1.online_ctr_mut().locked_in_characters[r.client_id as usize] = r.locked_in as i8;
            }
        }
        t if t == ServerMsg::Engine as u8 => {
            // GASMOX_CLIENT.cpp:441-462
            let r = server::MessageEngine::from_bytes(data);
            let driver_id = ps1.online_ctr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                };
                let engine = if r.engine_type > 3 { 3 } else { r.engine_type };
                ps1.online_ctr_mut().engine_type[slot as usize] = engine as i8;
                ps1.online_ctr_mut().locked_in_engines[r.client_id as usize] = r.locked_in as i8;
            }
        }
        t if t == ServerMsg::StartLoading as u8 => {
            ps1.online_ctr_mut().current_state = ClientState::LobbyStartLoading as i32;
        }
        t if t == ServerMsg::StartRace as u8 => {
            ps1.online_ctr_mut().current_state = ClientState::GameStartRace as i32;
        }
        t if t == ServerMsg::RaceData as u8 => {
            // GASMOX_CLIENT.cpp:482-578
            if ps1.online_ctr().current_state < ClientState::GameWaitForRace as i32 {
                return;
            }
            if ps1.read_u32(LOADING_STAGE) != 0xFFFFFFFF {
                return;
            }

            let r = server::EverythingKart::from_bytes(data);
            let driver_id = ps1.online_ctr().driver_id as u8;
            if r.client_id == driver_id {
                return;
            }
            let slot = if r.client_id < driver_id {
                r.client_id + 1
            } else {
                r.client_id
            } as usize;

            let mut curr = r.button_hold as u32;
            if (curr & 0x40) != 0 {
                curr &= !0x40;
                curr |= 0x400;
            }
            if (curr & 0x80) != 0 {
                curr &= !0x80;
                curr |= 0x800;
            }

            let prev = state.previous_button[slot] as u32;
            state.previous_button[slot] = curr as i32;

            let gp_addr = GAMEPAD_BASE + (slot as u32 * 0x50);
            ps1.write_u32(gp_addr, curr);
            ps1.write_u32(gp_addr + 0x4, !prev & curr);
            ps1.write_u32(gp_addr + 0x8, prev & !curr);
            ps1.write_u32(gp_addr + 0xC, prev);

            let psx_ptr = (ps1.read_u32(PSX_POINTER + (slot as u32 * 4)) & 0xFFFFFF) as u32;
            ps1.write_u32(psx_ptr + 0x2d4, ((r.position_x as i32) * 256) as u32);
            ps1.write_u32(psx_ptr + 0x2d8, ((r.position_y as i32) * 256) as u32);
            ps1.write_u32(psx_ptr + 0x2dc, ((r.position_z as i32) * 256) as u32);

            let angle = (r.kart_rotation1 as u32) | ((r.kart_rotation2 as u32) << 5);
            ps1.write_u16(psx_ptr + 0x39a, (angle & 0xFFF) as u16);

            if r.reserves {
                ps1.write_u16(psx_ptr + 0x3e2, 200);
            }
            ps1.write_u16(psx_ptr + 0x30, r.wumpa as u16);
        }
        t if t == ServerMsg::Weapon as u8 => {
            // GASMOX_CLIENT.cpp:581-598
            let r = server::MessageWeapon::from_bytes(data);
            let driver_id = ps1.online_ctr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                };
                ps1.online_ctr_mut().shoot[slot as usize].now = 1;
                ps1.online_ctr_mut().shoot[slot as usize].weapon = r.weapon;
                ps1.online_ctr_mut().shoot[slot as usize].juiced = r.juiced as u8;
                ps1.online_ctr_mut().shoot[slot as usize].flags = r.flags;
            }
        }
        t if t == ServerMsg::Warpclock as u8 => {
            // GASMOX_CLIENT.cpp:600-613
            let r = server::MessageWarpclock::from_bytes(data);
            state.previous_warpclock = r.warp_clock as i32;
            if ps1.online_ctr().warpclock != r.warp_clock {
                ps1.online_ctr_mut().warpclock = r.warp_clock;
            }
        }
        t if t == ServerMsg::FinishTimer as u8 => {
            // GASMOX_CLIENT.cpp:614-624
            let r = server::MessageFinishTimer::from_bytes(data);
            if r.finish_timer as i32 != state.previous_finish_timer {
                ps1.online_ctr_mut().finish_race_timer = r.finish_timer;
                state.previous_finish_timer = r.finish_timer as i32;
            }
        }
        t if t == ServerMsg::EndRace as u8 => {
            // GASMOX_CLIENT.cpp:625-660
            let r = server::MessageEndRace::from_bytes(data);
            let driver_id = ps1.online_ctr().driver_id as u8;
            if r.client_id == driver_id {
                return;
            }
            let slot = if r.client_id < driver_id {
                r.client_id + 1
            } else {
                r.client_id
            } as usize;

            if state.square_delay[slot] == 0 {
                state.square_delay[slot] = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            }

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("SystemTime before UNIX_EPOCH")
                .as_secs();
            if now - state.square_delay[slot] >= 3 {
                let gp_addr = GAMEPAD_BASE + (slot as u32 * 0x50);
                ps1.write_u32(gp_addr, 0x20);
                ps1.write_u32(gp_addr + 0x4, 0);
                ps1.write_u32(gp_addr + 0x8, 0);
                ps1.write_u32(gp_addr + 0xC, 0x20);
            }

            let ended = ps1.online_ctr().drivers_ended_count as usize;
            ps1.online_ctr_mut().race_stats[ended].slot = slot as i32;
            ps1.online_ctr_mut().race_stats[ended].final_time = r.course_time;
            ps1.online_ctr_mut().race_stats[ended].best_lap = r.lap_time;
            ps1.online_ctr_mut().drivers_ended_count += 1;
        }
        _ => {}
    }
}

// - StatePC_Launch_EnterPID → línea 799
// - StatePC_Launch_PickServer → línea 829
// - StatePC_Launch_PickRoom → línea 1119
// - StatePC_Launch_Error → línea 1112
// - StatePC_Launch_EnterPassword → línea 1156
// - StatePC_Lobby_AssignRole → línea 1182
// - StatePC_Lobby_HostTrackPick → línea 1215
// - StatePC_Lobby_SpecialPick → línea 1250
// - StatePC_Lobby_GuestTrackWait → línea 1285
// - StatePC_Lobby_CharacterPick → línea 1294
// - StatePC_Lobby_EnginePick → línea 1338
// - StatePC_Lobby_WaitForLoading → línea 1367
// - StatePC_Lobby_StartLoading → línea 1377
// - StatePC_Game_WaitForRace → línea 1453
// - StatePC_Game_StartRace → línea 1481
// - StatePC_Game_EndRace → línea 1622
// - ClientState[] (function pointer array) → línea 1685
// - afktimer() → línea 1749
// - DisconSELECT() → línea 753
// - SendEverything() → línea 1384

use std::io::Write;
use std::net::ToSocketAddrs;

use rusty_enet as enet;

use crate::{
    enet::EnetClient,
    protocol::{
        ClientState, DRIVER_BESTLAP_OFFSET, DRIVER_COURSE_OFFSET, GASMOXIAN_VER, GameMode,
        LOBBY_LEVEL_ID, MAX_NUM_PLAYERS, NAME_LENGTH, ServerMsg, client, server,
    },
    ps1mem::{
        ADDR_CHARACTER_ID, ADDR_CHEATS, ADDR_GAME_MODE, ADDR_GAMEPAD_BASE, ADDR_LOADING_STAGE,
        ADDR_PSX_PTR, Ps1Mem,
    },
    servers::SERVERS,
};

// all these variables are everywhere in the original code
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

pub type StateFn = fn(&Ps1Mem, &mut EnetClient, &mut GameState);

// mods/Windows/Gasmoxian/Network_PC/GClient/GASMOX_CLIENT.cpp:1685-1702
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

fn launch_enter_pid(ps1: &Ps1Mem, _net: &mut EnetClient, _state: &mut GameState) {
    if ps1.octr().is_booted_ps1 == 0 {
        return;
    }

    stop_animation();
    println!("Client: Waiting to connect to a server...");

    ps1.octr_mut().current_state = ClientState::LaunchPickServer as i32;
}

fn launch_pick_server(ps1: &Ps1Mem, _net: &mut EnetClient, state: &mut GameState) {
    // GASMOX_CLIENT.cpp:829-1110
    let level_id = ps1.read_u32(ADDR_GAME_MODE.wrapping_add(0x1a10)) as i32;
    if level_id != LOBBY_LEVEL_ID {
        return;
    }

    let loading = ps1.read_u32(ADDR_LOADING_STAGE);
    if loading != 0xFFFFFFFF {
        return;
    }

    let server_country = {
        let octr = ps1.octr();
        if octr.server_lock_in1 == 0 {
            return;
        }
        octr.server_country as usize
    };

    stop_animation();

    if server_country >= SERVERS.len() {
        println!("Client: Private server not yet implemented...");
        return;
    }

    let server = &SERVERS[server_country];
    let addr_str = format!("{}:{}", server.address, server.port);
    let addr = match addr_str.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => {
                println!(
                    "Error: Could not resolve server address: {}",
                    server.address
                );
                return;
            }
        },
        Err(e) => {
            println!(
                "Error: Failed to resolve server address: {} ({})",
                server.address, e
            );
            return;
        }
    };

    // Store connection info in state for main loop to use
    state.static_server_id = server_country as i32;
    state.server_addr = Some(addr);

    println!(
        "Client: Ready to connect to \"{}\" [{}]...",
        server.address,
        addr.ip()
    );
    ps1.octr_mut().current_state = ClientState::LaunchPickRoom as i32;
}

fn launch_pick_room(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    // room not updating bug still happens if the number is not 60, i didnt tried 30 anyways
    state.count_frame += 1;
    if state.count_frame == 60 {
        state.count_frame = 0;
        net.send_reliable(&client::MessageRoom { room: 0xFF }.to_bytes());
    }

    if ps1.octr().server_lock_in2 == 0 {
        state.connection_attempt = 0;
        return;
    }

    if state.connection_attempt == 1 {
        return;
    }
    state.connection_attempt = 1;
    let room = ps1.octr().server_room;
    net.send_reliable(&client::MessageRoom { room }.to_bytes());
    ps1.octr_mut().auto_retry_join_room_index = -1;
}

fn launch_error(_ps1: &Ps1Mem, _net: &mut EnetClient, _state: &mut GameState) {
    // do nothing
}

fn launch_enter_password(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    if state.password_sent {
        return;
    }

    // keep alive via Enet ping to prevent timeout
    state.count_frame += 1;
    if state.count_frame >= 60 {
        state.count_frame = 0;
        net.ping();
    }

    if ps1.octr().password_entered[7] == 0 {
        return;
    }

    let mut seq = [0u8; 8];
    for i in 0..8 {
        seq[i] = ps1.octr().room_password_sequence[i];
    }
    net.send_reliable(&client::MessagePassword { seq }.to_bytes());
    state.password_sent = true;
}

fn lobby_assign_role(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    state.connection_attempt = 0;
    state.count_frame = 0;

    if ps1.octr().driver_id > 0 {
        return; // guest: do nothing
    }
    if ps1.octr().r_type_locked == 0 {
        return;
    }

    stop_animation();
    let room_type = ps1.octr().room_type;
    let r_type_locked = ps1.octr().r_type_locked;
    print!("Client: Sending room type (");
    match room_type {
        1 => print!("TOURNAMENT"),
        2 => print!("PASSWORD"),
        _ => print!("NORMAL"),
    }
    println!(")...");

    if room_type == 2 {
        let mut seq = [0u8; 8];
        for i in 0..8 {
            seq[i] = ps1.octr().room_password_sequence[i];
        }
        net.send_reliable(
            &client::MessageRoomTypePassword {
                room_type,
                r_type_locked,
                seq,
            }
            .to_bytes(),
        );
    } else {
        net.send_reliable(
            &client::MessageRoomType {
                room_type,
                r_type_locked,
            }
            .to_bytes(),
        );
    }
}

fn lobby_host_track_pick(ps1: &Ps1Mem, net: &mut EnetClient, _state: &mut GameState) {
    let (lap_id, track_id) = {
        let octr = ps1.octr();
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

    stop_animation();
    println!("Client: Sending track to the server...");
    net.send_reliable(&client::MessageTrack { track_id, lap_id }.to_bytes());
    ps1.octr_mut().current_state = ClientState::LobbySpecialPick as i32;
}

fn lobby_special_pick(ps1: &Ps1Mem, net: &mut EnetClient, _state: &mut GameState) {
    let mut gamemodes = [false; 18];
    {
        let octr = ps1.octr();
        if octr.locked_in_special == 0 {
            return;
        }
        for i in 0..18 {
            gamemodes[i] = octr.gamemodes[i];
        }
        gamemodes[GameMode::Normal as usize] = true;
    }

    stop_animation();
    println!("Client: Sending gamemodes to the server...");
    net.send_reliable(&client::MessageSpecial { gamemodes }.to_bytes());
    ps1.octr_mut().current_state = ClientState::LobbyCharacterPick as i32;
}

fn lobby_guest_track_wait(_ps1: &Ps1Mem, _net: &mut EnetClient, state: &mut GameState) {
    state.previous_character_id = -1;
    state.previous_bool_locked_in = -1;
    state.previous_enginetype = -1;
    state.previous_bool_locked_in_engine = -1;
}

fn lobby_character_pick(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    let character_id = ps1.read_u8(ADDR_CHARACTER_ID) as i32;
    let bool_locked_in = ps1.octr().locked_in_characters[ps1.octr().driver_id as usize] as i32;

    if state.previous_character_id != character_id
        || state.previous_bool_locked_in != bool_locked_in
    {
        state.previous_character_id = character_id;
        state.previous_bool_locked_in = bool_locked_in;
        net.send_reliable(
            &client::MessageCharacter {
                character_id: character_id as u8,
                bool_locked_in: bool_locked_in != 0,
            }
            .to_bytes(),
        );
    }

    if bool_locked_in != 0 {
        ps1.octr_mut().current_state = ClientState::LobbyEnginePick as i32;
    }
}

fn lobby_engine_pick(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    let engine_type = ps1.octr().engine_type[0] as i32; // slot is not the same than octr().driver_id. 
    let bool_locked_in = ps1.octr().locked_in_engines[ps1.octr().driver_id as usize] as i32;
    if state.previous_enginetype != engine_type
        || state.previous_bool_locked_in_engine != bool_locked_in
    {
        state.previous_enginetype = engine_type;
        state.previous_bool_locked_in_engine = bool_locked_in;
        net.send_reliable(
            &client::MessageEngine {
                enginetype: engine_type as u8,
                bool_locked_in: bool_locked_in != 0,
            }
            .to_bytes(),
        );
    }

    if bool_locked_in != 0 {
        state.lock_engine_and_character = false;
        ps1.octr_mut().current_state = ClientState::LobbyWaitForLoading as i32;
    }
}

fn lobby_wait_for_loading(_ps1: &Ps1Mem, _net: &mut EnetClient, _state: &mut GameState) {
    // if recv message to start loading, change state to StartLoading, this check happens in ProcessNewMessages
}

fn lobby_start_loading(ps1: &Ps1Mem, _net: &mut EnetClient, state: &mut GameState) {
    ps1.octr_mut().finish_race_timer = 0;
    state.already_sent_start_race = 0;
    state.already_sent_end_race = 0;
}

fn game_wait_for_race(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    let game_mode = ps1.read_u32(ADDR_GAME_MODE);

    if state.already_sent_start_race == 0 && (game_mode & 0x40) == 0 {
        stop_animation();
        println!("Client: Gasmoxian race in progress...");
        state.already_sent_start_race = 1;
        net.send_reliable(&client::Header::to_bytes());
    }
    send_everything(ps1, net, state);
}

fn game_start_race(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    send_everything(ps1, net, state);
    // Demo camera mode
    if ps1.octr().gamemodes[GameMode::DemoCamera as usize] {
        let level_id = ps1.read_u32(ADDR_GAME_MODE.wrapping_add(0x1a10));
        if level_id < 18 {
            ps1.write_u16(0x80098028, 0x20);
        }
    }

    let warpclock = ps1.octr().warpclock as i32;

    // Warpclock cooldown logic (GASMOX_CLIENT.cpp:1500-1551)
    if state.send_warpclock == 0 && state.warpclock_delay == 0.0 {
        if warpclock != state.previous_warpclock {
            net.send_reliable(
                &client::MessageWarpclock {
                    warpclock: warpclock as u8,
                }
                .to_bytes(),
            );
            state.send_warpclock = 1;
            state.previous_warpclock = warpclock;
            state.warpclock_delay = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
        }
    }

    if state.send_warpclock != 0 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        state.timers[0] = now - state.warpclock_delay;
        if state.timers[0] >= 50.0 {
            if ps1.octr().warpclock != 0 {
                net.send_reliable(&client::MessageWarpclock { warpclock: 0 }.to_bytes());
            }
            state.send_warpclock = 0;
            state.warpclock_delay = 0.0;
            state.timers[0] = 0.0;
        }
    }

    // Calculate disconnected/active/required players (GASMOX_CLIENT.cpp:1564-1618)
    let (_, finish_race_timer) = {
        let octr = ps1.octr();
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

        if octr.drivers_ended_count as i32 == required
            && required != 0
            && state.previous_finish_timer != 30
        {
            let timer: u8 = if state.extra_laps != 0 { 60 } else { 30 };
            ps1.octr_mut().finish_race_timer = timer;
            state.previous_finish_timer = 30;
        }

        (octr.drivers_ended_count, octr.finish_race_timer)
    };

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

fn game_end_race(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    if state.already_sent_end_race == 0 {
        state.already_sent_end_race = 1;

        let psx_ptr = (ps1.read_u32(ADDR_PSX_PTR) & 0xFFFFFF) as u32;
        let course_time = ps1.read_u32(psx_ptr + DRIVER_COURSE_OFFSET);
        let best_lap = ps1.read_u32(psx_ptr + DRIVER_BESTLAP_OFFSET);

        net.send_reliable(
            &client::MessageEndRace {
                course_time: course_time as i32,
                lap_time: best_lap as i32,
            }
            .to_bytes(),
        );

        let ended = ps1.octr().drivers_ended_count as usize;
        ps1.octr_mut().race_stats[ended].slot = 0;
        ps1.octr_mut().race_stats[ended].final_time = course_time as i32;
        ps1.octr_mut().race_stats[ended].best_lap = best_lap as i32;
        ps1.octr_mut().drivers_ended_count += 1;
    }

    if state.already_sent_end_race != 0 {
        // GASMOX_CLIENT.cpp:1654-1674
        let (finish_race_timer, needs_send) = {
            let octr = ps1.octr();
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
                ps1.octr_mut().finish_race_timer = timer;
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

fn send_everything(ps1: &Ps1Mem, net: &mut EnetClient, _state: &mut GameState) {
    // position
    let hold_raw = ps1.read_u32(ADDR_GAMEPAD_BASE + 0x10);

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
    let psx_pointer = (ps1.read_u32(ADDR_PSX_PTR) & 0xFFFFFF) as u32;

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
        kart_rot1: (angle & 0x1f) as u8, // angle_bit_5
        kart_rot2: (angle >> 5) as u8,   // angle_top_8
        button_hold: hold as u8,
        pos_x: position_x,
        pos_y: position_y,
        pos_z: position_z,
    };

    net.send_unsequenced(&kart.to_bytes());

    let weapon_data = {
        let octr = ps1.octr();
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
        ps1.octr_mut().shoot[0].now = 0;
        let w = client::MessageWeapon {
            juiced,
            flags,
            weapon,
        };
        net.send_reliable(&w.to_bytes());
    }
}

// GASMOX_CLIENT.cpp:2084-2093
pub fn frame_stall(ps1: &Ps1Mem) {
    while ps1.octr().ready_to_send == 0 {
        std::thread::sleep(std::time::Duration::from_micros(1));
    }
    ps1.octr_mut().ready_to_send = 0;
}

// GASMOX_CLIENT.cpp:753-776
pub fn discon_select(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    let hold = ps1.read_u32(ADDR_GAMEPAD_BASE + 0x10);
    if (hold & 0x2000) != 0 {
        stop_animation();
        println!("Client: Disconnected (ID: DSELECT)...");
        net.disconnect_now();
        state.lock_engine_and_character = false;
        ps1.octr_mut().auto_retry_join_room_index = -1;
        ps1.octr_mut().room_type = 0;
        ps1.octr_mut().r_type_locked = 0;
        ps1.octr_mut().current_state = -1;
    }
}

// GASMOX_CLIENT.cpp:1749-1780
pub fn afk_timer(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
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
        stop_animation();
        println!("Client: Kicked, reason: AFK...");
        net.disconnect_now();
        state.lock_engine_and_character = false;
        ps1.octr_mut().room_type = 0;
        ps1.octr_mut().r_type_locked = 0;
        ps1.octr_mut().current_state = -1;
        state.time_start = 0.0;
    }
}

fn stop_animation() {
    print!("\r   \r"); // limpia el spinner
    std::io::stdout().flush().ok();
}

// GASMOX_CLIENT.cpp:668-712
pub fn process_new_messages(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    while let Some(event) = net.poll().unwrap() {
        match event {
            enet::Event::Receive { packet, .. } => {
                process_receive_event(ps1, net, state, packet.data());
            }
            enet::Event::Disconnect { .. } => {
                println!("\nClient: Connection Dropped (Server Full or Server Offline)...");
                state.password_sent = false;
                ps1.octr_mut().current_state = -1;
            }
            _ => {}
        }
    }
}

// GASMOX_CLIENT.cpp:152-665
fn process_receive_event(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState, data: &[u8]) {
    let msg_type = data[0] & 0x0F;
    match msg_type {
        t if t == ServerMsg::Rooms as u8 => {
            // GASMOX_CLIENT.cpp:161-207
            let r = server::MessageRooms::from_bytes(data);

            ps1.octr_mut().pc_version = GASMOXIAN_VER;
            ps1.octr_mut().server_version = r.version as i32;

            if r.version != GASMOXIAN_VER as u16 {
                stop_animation();
                println!(
                    "Client: Version mismatch! Server={}, Client={}",
                    r.version, GASMOXIAN_VER
                );
                ps1.octr_mut().current_state = ClientState::LaunchError as i32;
                return;
            }

            if ps1.octr().psx_version != GASMOXIAN_VER {
                stop_animation();
                println!("Client: PSX version mismatch!");
                ps1.octr_mut().current_state = ClientState::LaunchError as i32;
                return;
            }

            let curr_state = ps1.octr().current_state;
            if curr_state == ClientState::LaunchEnterPassword as i32
                || curr_state >= ClientState::LobbyAssignRole as i32
            {
                return;
            }

            ps1.octr_mut().server_lock_in2 = 0;
            ps1.octr_mut().room_count = r.num_rooms;
            for i in 0..16 {
                ps1.octr_mut().client_count[i] = r.client_count[i];
            }
        }
        t if t == ServerMsg::RoomType as u8 => {
            // GASMOX_CLIENT.cpp:209-224
            let r = server::MessageRoomType::from_bytes(data);
            ps1.octr_mut().room_type = r.room_type;
            if r.room_type == 2
                && ps1.octr().r_type_locked == 0
                && ps1.octr().current_state == ClientState::LaunchPickRoom as i32
            {
                state.password_sent = false;
                ps1.octr_mut().current_state = ClientState::LaunchEnterPassword as i32;
            }
        }
        t if t == ServerMsg::PasswordRejected as u8 => {
            // GASMOX_CLIENT.cpp:226-242
            stop_animation();
            println!("\nClient: Wrong password. Returning to room list.");
            net.disconnect_now();
            ps1.octr_mut().room_type = 0;
            ps1.octr_mut().r_type_locked = 0;
            state.lock_engine_and_character = false;
            ps1.octr_mut().auto_retry_join_room_index = -1;
            for i in 0..8 {
                ps1.octr_mut().password_entered[i] = 0;
                ps1.octr_mut().room_password_sequence[i] = 0;
            }
            state.password_sent = false;
            ps1.octr_mut().current_state = ClientState::LaunchPickRoom as i32;
        }
        t if t == ServerMsg::NewClient as u8 => {
            // GASMOX_CLIENT.cpp:244-300
            let r = server::MessageClientStatus::from_bytes(data);
            state.password_sent = false;
            ps1.octr_mut().driver_id = r.client_id;
            ps1.octr_mut().driver_count = r.num_clients;
            ps1.octr_mut().locked_in_lap = 0;
            ps1.octr_mut().locked_in_level = 0;
            ps1.octr_mut().locked_in_engine = 0;
            ps1.octr_mut().locked_in_special = 0;
            ps1.octr_mut().lap_id = 0;
            ps1.octr_mut().special = 0;
            ps1.octr_mut().level_id = 0;
            ps1.octr_mut().locked_in_character = 0;
            ps1.octr_mut().drivers_ended_count = 0;
            ps1.octr_mut().finish_race_timer = 0;
            ps1.octr_mut().warpclock = 0;
            state.lock_engine_and_character = false;
            state.already_sended = 0;
            state.send_warpclock = 0;
            state.previous_warpclock = -1;
            state.previous_special = -1;
            state.previous_finish_timer = -1;
            state.extra_laps = 0;

            if ps1.octr().server_room == 15 {
                println!("\n EASTER EGG: SAFFI FIRE UNLOCKED IN THIS ROOM \n");
            }

            // Zero out all arrays
            for i in 0..MAX_NUM_PLAYERS {
                ps1.octr_mut().locked_in_characters[i] = 0;
                ps1.octr_mut().locked_in_engines[i] = 0;
                for j in 0..=NAME_LENGTH {
                    ps1.octr_mut().name_buffer[i][j] = 0;
                }
                ps1.octr_mut().race_stats[i].slot = 0;
                ps1.octr_mut().race_stats[i].final_time = 0;
                ps1.octr_mut().race_stats[i].best_lap = 0;
            }
            state.square_delay = [0; MAX_NUM_PLAYERS];
            for i in 0..8 {
                ps1.octr_mut().password_entered[i] = 0;
                ps1.octr_mut().room_password_sequence[i] = 0;
            }

            // Send name to server
            let name_buf = state.name;
            for i in 0..NAME_LENGTH {
                ps1.octr_mut().name_buffer[0][i] = name_buf[i];
            }
            ps1.octr_mut().name_buffer[0][NAME_LENGTH] = 0;

            let mut name = [0u8; 12];
            for i in 0..NAME_LENGTH {
                name[i] = name_buf[i];
            }
            net.send_reliable(&client::MessageName { name }.to_bytes());

            ps1.octr_mut().current_state = ClientState::LobbyAssignRole as i32;
        }
        t if t == ServerMsg::Name as u8 => {
            // GASMOX_CLIENT.cpp:302-329
            let r = server::MessageName::from_bytes(data);
            let driver_id = ps1.octr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                } as usize;
                ps1.octr_mut().driver_count = r.num_clients;
                for i in 0..NAME_LENGTH {
                    ps1.octr_mut().name_buffer[slot][i] = r.name[i];
                }
                ps1.octr_mut().name_buffer[slot][NAME_LENGTH] = 0;

                // Handle disconnection — force SQUARE if name starts with 0
                if r.name[0] == 0
                    || ps1.octr().current_state <= ClientState::LobbyWaitForLoading as i32
                {
                    let gp_addr = ADDR_GAMEPAD_BASE + (slot as u32 * 0x50);
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
            ps1.octr_mut().level_id = r.track_id;
            ps1.octr_mut().current_state = ClientState::LobbySpecialPick as i32;
        }
        t if t == ServerMsg::Special as u8 => {
            // GASMOX_CLIENT.cpp:363-410
            let r = server::MessageSpecial::from_bytes(data);
            let octr = ps1.octr_mut();
            // Copy all gamemode toggles
            for i in 0..18 {
                octr.gamemodes[i] = r.gamemodes[i];
            }
            octr.gamemodes[GameMode::Normal as usize] = true;

            // Apply cheat effects
            let mut cheats = ps1.read_u32(ADDR_CHEATS);
            cheats &= !(0x100000 | 0x80000 | 0x400 | 0x400000 | 0x8000000 | 0x10000);
            if octr.gamemodes[GameMode::IcyTracks as usize] {
                cheats |= 0x80000;
                println!("\n MODE: ICY TRACKS");
            }
            if octr.gamemodes[GameMode::RetroFueled as usize] {
                cheats |= 0x100000;
                println!("\n MODE: RETRO FUELED");
            }
            ps1.write_u32(ADDR_CHEATS, cheats);

            for i in 0..18 {
                if i != GameMode::IcyTracks as usize && i != GameMode::RetroFueled as usize {
                    if octr.gamemodes[i] {
                        println!(
                            "\n MODE: {} ENABLED",
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
                        );
                    }
                }
            }
            ps1.octr_mut().current_state = ClientState::LobbyCharacterPick as i32;
        }
        t if t == ServerMsg::Character as u8 => {
            // GASMOX_CLIENT.cpp:411-440
            let r = server::MessageCharacter::from_bytes(data);
            let driver_id = ps1.octr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                };
                ps1.write_u16(0x80086e84 + (slot as u32 * 2), r.character_id as u16);
                ps1.octr_mut().locked_in_characters[r.client_id as usize] = r.bool_locked_in as i8;
            }
        }
        t if t == ServerMsg::Engine as u8 => {
            // GASMOX_CLIENT.cpp:441-462
            let r = server::MessageEngine::from_bytes(data);
            let driver_id = ps1.octr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                };
                let engine = if r.enginetype > 3 { 3 } else { r.enginetype };
                ps1.octr_mut().engine_type[slot as usize] = engine as i8;
                ps1.octr_mut().locked_in_engines[r.client_id as usize] = r.bool_locked_in as i8;
            }
        }
        t if t == ServerMsg::StartLoading as u8 => {
            ps1.octr_mut().current_state = ClientState::LobbyStartLoading as i32;
        }
        t if t == ServerMsg::StartRace as u8 => {
            ps1.octr_mut().current_state = ClientState::GameStartRace as i32;
        }
        t if t == ServerMsg::RaceData as u8 => {
            // GASMOX_CLIENT.cpp:482-578
            if ps1.octr().current_state < ClientState::GameWaitForRace as i32 {
                return;
            }
            if ps1.read_u32(ADDR_LOADING_STAGE) != 0xFFFFFFFF {
                return;
            }

            let r = server::EverythingKart::from_bytes(data);
            let driver_id = ps1.octr().driver_id as u8;
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

            let gp_addr = ADDR_GAMEPAD_BASE + (slot as u32 * 0x50);
            ps1.write_u32(gp_addr, curr);
            ps1.write_u32(gp_addr + 0x4, !prev & curr);
            ps1.write_u32(gp_addr + 0x8, prev & !curr);
            ps1.write_u32(gp_addr + 0xC, prev);

            let psx_ptr = (ps1.read_u32(ADDR_PSX_PTR + (slot as u32 * 4)) & 0xFFFFFF) as u32;
            ps1.write_u32(psx_ptr + 0x2d4, ((r.pos_x as i32) * 256) as u32);
            ps1.write_u32(psx_ptr + 0x2d8, ((r.pos_y as i32) * 256) as u32);
            ps1.write_u32(psx_ptr + 0x2dc, ((r.pos_z as i32) * 256) as u32);

            let angle = (r.kart_rot1 as u32) | ((r.kart_rot2 as u32) << 5);
            ps1.write_u16(psx_ptr + 0x39a, (angle & 0xFFF) as u16);

            if r.bool_reserves {
                ps1.write_u16(psx_ptr + 0x3e2, 200);
            }
            ps1.write_u16(psx_ptr + 0x30, r.wumpa as u16);
        }
        t if t == ServerMsg::Weapon as u8 => {
            // GASMOX_CLIENT.cpp:581-598
            let r = server::MessageWeapon::from_bytes(data);
            let driver_id = ps1.octr().driver_id as u8;
            if r.client_id != driver_id {
                let slot = if r.client_id < driver_id {
                    r.client_id + 1
                } else {
                    r.client_id
                };
                ps1.octr_mut().shoot[slot as usize].now = 1;
                ps1.octr_mut().shoot[slot as usize].weapon = r.weapon;
                ps1.octr_mut().shoot[slot as usize].juiced = r.juiced as u8;
                ps1.octr_mut().shoot[slot as usize].flags = r.flags;
            }
        }
        t if t == ServerMsg::Warpclock as u8 => {
            // GASMOX_CLIENT.cpp:600-613
            let r = server::MessageWarpclock::from_bytes(data);
            state.previous_warpclock = r.warpclock as i32;
            if ps1.octr().warpclock != r.warpclock {
                ps1.octr_mut().warpclock = r.warpclock;
            }
        }
        t if t == ServerMsg::FinishTimer as u8 => {
            // GASMOX_CLIENT.cpp:614-624
            let r = server::MessageFinishTimer::from_bytes(data);
            if r.finish_timer as i32 != state.previous_finish_timer {
                ps1.octr_mut().finish_race_timer = r.finish_timer;
                state.previous_finish_timer = r.finish_timer as i32;
            }
        }
        t if t == ServerMsg::EndRace as u8 => {
            // GASMOX_CLIENT.cpp:625-660
            let r = server::MessageEndRace::from_bytes(data);
            let driver_id = ps1.octr().driver_id as u8;
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
                .unwrap()
                .as_secs();
            if now - state.square_delay[slot] >= 3 {
                let gp_addr = ADDR_GAMEPAD_BASE + (slot as u32 * 0x50);
                ps1.write_u32(gp_addr, 0x20);
                ps1.write_u32(gp_addr + 0x4, 0);
                ps1.write_u32(gp_addr + 0x8, 0);
                ps1.write_u32(gp_addr + 0xC, 0x20);
            }

            let ended = ps1.octr().drivers_ended_count as usize;
            ps1.octr_mut().race_stats[ended].slot = slot as i32;
            ps1.octr_mut().race_stats[ended].final_time = r.course_time;
            ps1.octr_mut().race_stats[ended].best_lap = r.lap_time;
            ps1.octr_mut().drivers_ended_count += 1;
        }
        _ => {}
    }
}

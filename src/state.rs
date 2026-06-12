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

use crate::{
    enet::EnetClient,
    protocol::{CgEverythingKart, CgMessageWeapon, ClientState, GameMode, MAX_NUM_PLAYERS, NAME_LENGTH},
    ps1mem::{ADDR_CHARACTER_ID, ADDR_GAME_MODE, ADDR_GAMEPAD_BASE, ADDR_PSX_PTR, Ps1Mem},
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

fn launch_pick_server(_ps1: &Ps1Mem, _net: &mut EnetClient, _state: &mut GameState) {
    todo!()
}

fn launch_pick_room(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    // room not updating bug still happens if the number is not 60, i didnt tried 30 anyways
    state.count_frame += 1;
    if state.count_frame == 60 {
        state.count_frame = 0;
        // TODO: URGENTE!!! enviar CG_JOINROOM con room=0xFF como ping
    }

    if ps1.octr().server_lock_in2 == 0 {
        state.connection_attempt = 0;
        return;
    }

    if state.connection_attempt == 1 {
        return;
    }
    state.connection_attempt = 1;
    // To do: enviar CG_JOINROOM con la sala elegida
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

    // TODO: URGENTE!!! construir CG_MessagePassword con bitfields y enviar
    // Por ahora queda pendiente hasta que implementemos protocol.rs
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
    print!("Client: Sending room type (");
    match ps1.octr().room_type {
        1 => print!("TOURNAMENT"),
        2 => print!("PASSWORD"),
        _ => print!("NORMAL"),
    }
    println!(")...");

    // TODO: URGENTE!!! enviar CG_ROOMTYPE o CG_MessageRoomTypePassword según room_type
}

fn lobby_host_track_pick(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    // locked_in_lap gets set after locked_in_level is already set
    if ps1.octr().locked_in_lap == 0 {
        return;
    }

    // TODO: URGENTE!! calcular numLaps, escribir en PS1, enviar CG_TRACK

    stop_animation();
    println!("Client: Sending track to the server...");
    ps1.octr_mut().current_state = ClientState::LobbySpecialPick as i32;
}

fn lobby_special_pick(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    if ps1.octr().locked_in_special == 0 {
        return;
    }
    // TODO: URGENTE!! armar struct con gamemodes y enviar CG_SPECIAL

    stop_animation();
    println!("Client: Sending gamemodes to the server...");
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
        // TODO: URGENTE!!! armar CG_MessageCharacter y enviar
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
        // TODO: URGENTE!!! armar CG_MessageEngine y enviar
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

fn game_wait_for_race(ps1: &Ps1Mem, _net: &mut EnetClient, state: &mut GameState) {
    let game_mode = ps1.read_u32(ADDR_GAME_MODE);

    // only send once or after camara fly-in is done
    // TODO: Maybe unhardcode 0x40
    if state.already_sent_start_race == 0 && (game_mode & 0x40) == 0 {
        stop_animation();
        println!("Client: Gasmoxian race in progress...");
        state.already_sent_start_race = 1;
        // TODO: URGENTE!!! enviar CG_STARTRACE
    }
    // TODO: URGENTE!!!: send_everything(ps1, net, state);
}

fn game_start_race(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    // TODO: URGENTE!!! send_everything(ps1, net, state);
    // Demo camera mode
    if ps1.octr().gamemodes[GameMode::DemoCamera as usize] {
        let level_id = ps1.read_u32(ADDR_GAME_MODE.wrapping_add(0x1a10));
        if level_id < 18 {
            // TODO: Maybe unhardcode 0x80098028 and 0x20
            ps1.write_u16(0x80098028, 0x20);
        }
    }

    // TODO: URGENTE!!!: warpclock logic
    // TODO: URGENTE!!!: calculate disconnected/active/required players
    // TODO: URGENTE!!!: finish timer logic
}

fn game_end_race(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    if state.already_sent_end_race == 0 {
        state.already_sent_end_race = 1;
        // TODO: URGENTE!!! leer tiempos de PS1 y enviar CG_ENDRACE
        let mut octr = ps1.octr_mut();
        // octr.race_stats[octr.num_drivers_ended as usize].slot = 0;
        // octr.race_stats[...].final_time = course_time;
        // octr.race_stats[...].best_lap = best_lap;
        // octr.num_drivers_ended += 1;
    }

    if state.already_sent_end_race != 0 {
        // TODO: URGENTE!!! calcular si todos terminaron y enviar CG_FINISHTIMER
    }
}

fn send_everything(ps1: &Ps1Mem, net: &mut EnetClient, state: &mut GameState) {
    // position
    let hold_raw = ps1.read_u32(ADDR_GAMEPAD_BASE + 0x10);

    // lossless compression, bottom byte is never used,
	// cause psx renders with 3 bytes, and top byte
	// is never used due to world scale (just pure luck)

    // ignore Circle/L2
    let mut hold = hold_raw & !0xC0;

    // put L1/R1 into one byte
    if (hold & 0x400) != 0 { hold |= 0x40; }
    if (hold & 0x800) != 0 { hold |= 0x80; }
    
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
    
    let kart = CgEverythingKart {
        wumpa,
        reserves: reserves > 200,
        kart_rot1: (angle & 0x1f) as u8, // angle_bit_5
        kart_rot2: (angle >> 5) as u8, // angle_top_8
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
            Some((octr.shoot[0].weapon, octr.shoot[0].juiced != 0, octr.shoot[0].flags))
        }
    };
    
    if let Some((weapon, juiced, flags)) = weapon_data {
        ps1.octr_mut().shoot[0].now = 0;
        let w = CgMessageWeapon { juiced, flags, weapon };
        net.send_reliable(&w.to_bytes());
    }
}

fn stop_animation() {
    print!("\r   \r"); // limpia el spinner
    std::io::stdout().flush().ok();
}

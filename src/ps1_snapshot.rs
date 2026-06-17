//! Frozen copy of PS1 shared memory and auxiliary reads.
//!
//! [`OnlineCtrSnapshot::capture`] reads every field from the mmap'd
//! [`OnlineCTR`] block plus several absolute PS1 addresses (gamepad,
//! kart position, cheats, loading stage). State functions receive
//! this snapshot instead of `&Ps1Memory` so they are deterministic
//! and I/O-free.

use crate::protocol::{MAX_NAME_LENGTH, MAX_NUM_PLAYERS, OnlineCTR, ShootSlot};
use crate::ps1_memory::{
    CHARACTER_ID, CHEATS, GAMEMODE, GAMEPAD_BASE, LOADING_STAGE, PSX_POINTER, Ps1Memory,
};

/// Frozen copy of the `OnlineCTR` block and other `Ps1Memory` reads.
///
/// Built once at the start of each main-loop iteration.
/// State functions receive this instead of `&Ps1Memory`.
#[derive(Clone, Debug)]
pub struct OnlineCtrSnapshot {
    pub current_state: i32,
    pub page_number: i8,
    pub count_press_x: u8,
    pub driver_count: u8,
    pub driver_id: u8,
    pub locked_in_lap: u8,
    pub locked_in_level: u8,
    pub lap_id: u8,
    pub level_id: u8,
    pub is_booted_ps1: u8,
    pub locked_in_character: u8,
    pub locked_in_engine: u8,
    pub room_count: u8,
    pub drivers_ended_count: u8,
    pub server_country: u8,
    pub server_room: u8,
    pub server_lock_in1: u8,
    pub server_lock_in2: u8,
    pub planet_lev: u8,
    pub client_busy: u8,
    pub locked_in_special: u8,
    pub special: u8,
    pub warpclock: u8,
    pub finish_race_timer: u8,
    pub client_count: [i8; 16],
    pub windows_client_sync: i8,
    pub locked_in_characters: [i8; MAX_NUM_PLAYERS],
    pub locked_in_engines: [i8; MAX_NUM_PLAYERS],
    pub engine_type: [i8; MAX_NUM_PLAYERS],
    pub name_buffer: [[u8; MAX_NAME_LENGTH + 1]; MAX_NUM_PLAYERS],
    pub psx_version: i32,
    pub pc_version: i32,
    pub server_version: i32,
    pub shoot: [ShootSlot; MAX_NUM_PLAYERS],
    pub frames_unsynced: i32,
    pub last_windows_client_sync: i8,
    pub ready_to_send: i32,
    pub auto_retry_join_room_index: i32,
    pub gamemodes: [bool; 18],
    pub room_type: u8,
    pub room_type_locked: u8,
    pub room_password_sequence: [u8; 8],
    pub password_entered: [u8; 8],

    /// Button hold value read from GAMEPAD_BASE + 0x10
    pub gamepad_hold: u32,
    /// PSX_POINTER & 0xFFFFFF, used to read kart position/rotation
    pub psx_pointer: u32,
    /// PSX_POINTER + (slot * 4) for each slot — other players' psx_pointers
    pub slot_psx_pointers: [u32; MAX_NUM_PLAYERS],
    /// LOADING_STAGE (0x8008d0f8) — active loading screen
    pub loading_stage: u32,
    /// GAMEMODE (0x80096b20) — active game mode
    pub game_mode: u32,
    /// level_id read from GAMEMODE + 0x1a10 (demo mode / country select)
    pub cutscene_level_id: i8,
    /// CHARACTER_ID (0x80086e84) — selected character
    pub character_id: u32,
    /// CHEATS (0x80096b28) — active cheat bits
    pub cheats: u32,
    /// Current time (seconds since epoch) for AFK timers, timeouts, etc.
    pub now_secs: f64,

    /// Kart position X (psx_ptr + 0x2d4) / 256
    pub kart_position_x: i16,
    /// Kart position Y (psx_ptr + 0x2d8) / 256
    pub kart_position_y: i16,
    /// Kart position Z (psx_ptr + 0x2dc) / 256
    pub kart_position_z: i16,
    /// Kart angle / direction (psx_ptr + 0x39a) & 0xfff
    pub kart_angle: u16,
    /// Collected wumpas (psx_ptr + 0x30)
    pub kart_wumpa: u8,
    /// Turbo reserves (psx_ptr + 0x3e2)
    pub kart_reserves: u16,
    /// Race time in ms (psx_ptr + 0x514)
    pub race_course_time: i32,
    /// Best lap in ms (psx_ptr + 0x63C)
    pub race_best_lap: i32,
}

impl OnlineCtrSnapshot {
    /// Builds a snapshot by reading everything at once from `Ps1Memory`.
    /// All reads happen sequentially; the emulator cannot mutate memory
    /// between two accesses (single `connect()` → multiple dereferences,
    /// but the time window is so short it is effectively atomic).
    pub fn capture(ps1: &Ps1Memory, now_secs: f64) -> Self {
        let ctr: &OnlineCTR = ps1.online_ctr();
        let psx_pointer = (ps1.read_u32(PSX_POINTER).unwrap_or(0) & 0xFFFFFF) as u32;
        let gamepad_hold = ps1.read_u32(GAMEPAD_BASE + 0x10).unwrap_or(0);
        let loading_stage = ps1.read_u32(LOADING_STAGE).unwrap_or(0xFFFFFFFF);
        let game_mode = ps1.read_u32(GAMEMODE).unwrap_or(0);
        let cutscene_level_id = ps1.read_u32(GAMEMODE.wrapping_add(0x1a10)).unwrap_or(0) as i8;
        let character_id = ps1.read_u32(CHARACTER_ID).unwrap_or(0);
        let cheats = ps1.read_u32(CHEATS).unwrap_or(0);
        let mut slot_psx_pointers = [0u32; MAX_NUM_PLAYERS];
        for slot in 0..MAX_NUM_PLAYERS {
            slot_psx_pointers[slot] =
                ps1.read_u32(PSX_POINTER + (slot as u32 * 4)).unwrap_or(0) & 0xFFFFFF;
        }

        // Race data (read dynamically via psx_pointer)
        let (kart_position_x, kart_position_y, kart_position_z) = if psx_pointer != 0 {
            (
                (ps1.read_u32(psx_pointer + 0x2d4).unwrap_or(0) / 256) as i16,
                (ps1.read_u32(psx_pointer + 0x2d8).unwrap_or(0) / 256) as i16,
                (ps1.read_u32(psx_pointer + 0x2dc).unwrap_or(0) / 256) as i16,
            )
        } else {
            (0, 0, 0)
        };
        let kart_angle = if psx_pointer != 0 {
            ps1.read_u16(psx_pointer + 0x39a).unwrap_or(0) & 0xfff
        } else {
            0
        };
        let kart_wumpa = if psx_pointer != 0 {
            ps1.read_u8(psx_pointer + 0x30).unwrap_or(0)
        } else {
            0
        };
        let kart_reserves = if psx_pointer != 0 {
            ps1.read_u16(psx_pointer + 0x3e2).unwrap_or(0)
        } else {
            0
        };
        let (race_course_time, race_best_lap) = if psx_pointer != 0 {
            (
                ps1.read_u32(psx_pointer + 0x514).unwrap_or(0) as i32,
                ps1.read_u32(psx_pointer + 0x63C).unwrap_or(0) as i32,
            )
        } else {
            (0, 0)
        };

        Self {
            current_state: ctr.current_state,
            page_number: ctr.page_number,
            count_press_x: ctr.count_press_x,
            driver_count: ctr.driver_count,
            driver_id: ctr.driver_id,
            locked_in_lap: ctr.locked_in_lap,
            locked_in_level: ctr.locked_in_level,
            lap_id: ctr.lap_id,
            level_id: ctr.level_id,
            is_booted_ps1: ctr.is_booted_ps1,
            locked_in_character: ctr.locked_in_character,
            locked_in_engine: ctr.locked_in_engine,
            room_count: ctr.room_count,
            drivers_ended_count: ctr.drivers_ended_count,
            server_country: ctr.server_country,
            server_room: ctr.server_room,
            server_lock_in1: ctr.server_lock_in1,
            server_lock_in2: ctr.server_lock_in2,
            planet_lev: ctr.planet_lev,
            client_busy: ctr.client_busy,
            locked_in_special: ctr.locked_in_special,
            special: ctr.special,
            warpclock: ctr.warpclock,
            finish_race_timer: ctr.finish_race_timer,
            client_count: ctr.client_count,
            windows_client_sync: ctr.windows_client_sync,
            locked_in_characters: ctr.locked_in_characters,
            locked_in_engines: ctr.locked_in_engines,
            engine_type: ctr.engine_type,
            name_buffer: ctr.name_buffer,
            psx_version: ctr.psx_version,
            pc_version: ctr.pc_version,
            server_version: ctr.server_version,
            shoot: ctr.shoot,
            frames_unsynced: ctr.frames_unsynced,
            last_windows_client_sync: ctr.last_windows_client_sync,
            ready_to_send: ctr.ready_to_send,
            auto_retry_join_room_index: ctr.auto_retry_join_room_index,
            gamemodes: ctr.gamemodes,
            room_type: ctr.room_type,
            room_type_locked: ctr.room_type_locked,
            room_password_sequence: ctr.room_password_sequence,
            password_entered: ctr.password_entered,
            gamepad_hold,
            psx_pointer,
            slot_psx_pointers,
            loading_stage,
            game_mode,
            cutscene_level_id,
            character_id,
            cheats,
            now_secs,
            kart_position_x,
            kart_position_y,
            kart_position_z,
            kart_angle,
            kart_wumpa,
            kart_reserves,
            race_course_time,
            race_best_lap,
        }
    }
}

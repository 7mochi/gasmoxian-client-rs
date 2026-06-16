use deku::{DekuRead, DekuWrite};
use num_derive::FromPrimitive;

pub mod client;
pub mod server;

pub const GASMOXIAN_VERSION: i8 = 3;
pub const MAX_NUM_PLAYERS_NORMAL: usize = 8;
pub const MAX_NUM_PLAYERS_TOURNAMENT: usize = 4;
pub const MAX_NUM_PLAYERS: usize = MAX_NUM_PLAYERS_NORMAL;
pub const MAX_NAME_LENGTH: usize = 11;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RaceStats {
    pub slot: i32,
    pub final_time: i32,
    pub best_lap: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShootSlot {
    pub juiced: u8,
    pub weapon: u8,
    pub flags: u8,
    pub now: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OnlineCTR {
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
    pub padding: i8,
    pub client_count: [i8; 16],
    pub windows_client_sync: i8,
    pub locked_in_characters: [i8; MAX_NUM_PLAYERS],
    pub locked_in_engines: [i8; MAX_NUM_PLAYERS],
    pub engine_type: [i8; MAX_NUM_PLAYERS],
    pub name_buffer: [[u8; MAX_NAME_LENGTH + 1]; MAX_NUM_PLAYERS],
    pub race_stats: [RaceStats; MAX_NUM_PLAYERS],
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
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum ClientState {
    LaunchEnterPid,
    LaunchPickServer,
    LaunchPickRoom,
    LaunchError,
    LaunchEnterPassword,
    LobbyAssignRole,
    LobbyHostTrackPick,
    LobbySpecialPick,
    LobbyGuestTrackWait,
    LobbyCharacterPick,
    LobbyEnginePick,
    LobbyWaitForLoading,
    LobbyStartLoading,
    GameWaitForRace,
    GameStartRace,
    GameEndRace,
}

#[derive(Debug, Clone, Copy, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
#[repr(u8)]
pub enum ClientMessage {
    JoinRoom = 0,
    RoomType,
    Name,
    Track,
    Special,
    Character,
    Engine,
    StartRace,
    RaceData,
    Weapon,
    Warpclock,
    FinishTimer,
    EndRace,
    Password,
}

#[repr(u8)]
#[derive(FromPrimitive, Debug)]
pub enum ServerMessage {
    Rooms,
    RoomType,
    RoomTypeRejected,
    NewClient,
    Name,
    Track,
    Special,
    Character,
    Engine,
    StartLoading,
    StartRace,
    RaceData,
    Weapon,
    Warpclock,
    FinishTimer,
    EndRace,
    ServerClosed,
    PasswordRejected,
}

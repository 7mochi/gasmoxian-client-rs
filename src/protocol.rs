pub const GASMOXIAN_VER: i32 = 3; // TODO: maybe use a smaller type for this?

// INTRO_OXIDE
pub const LOBBY_LEVEL_ID: i32 = 38; // TODO: maybe use a smaller type for this?

pub const NAME_LENGTH: usize = 11; // TODO: maybe use a smaller type for this?

pub const MAX_NUM_PLAYERS_NORMAL: usize = 8; // TODO: maybe use a smaller type for this?
pub const MAX_NUM_PLAYERS_TOURNAMENT: usize = 4; // TODO: maybe use a smaller type for this?
pub const MAX_NUM_PLAYERS: usize = MAX_NUM_PLAYERS_NORMAL; // TODO: maybe use a smaller type for this?

pub const DEFAULT_IP: &str = "127.0.0.1"; // the default IP address we want to use for private lobbies
pub const IP_ADDRESS_SIZE: usize = 16; // assuming IPv4 (which is "xxx.xxx.xxx.xxx" + '\0')
// the port number as a string (0-65535 + '\0')
pub const PORT_SIZE: usize = 6; // TODO: maybe use an exact type for this?

pub const SHMEM_SIZE: usize = 0x800000; // TODO: maybe use an exact type for this?
pub const OCTR_OFFSET: usize = 0x8000C000 & 0xFFFFFF; // TODO: maybe use an exact type for this?

// decompile/General/AltMods/Gasmoxian/global.h
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RaceStats {
    pub slot: i32,
    pub final_time: i32,
    pub best_lap: i32,
}

// mods/Windows/Gasmoxian/Network_PC/GClient/GASMOX_CLIENT.cpp
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Gamepad {
    pub unknown_0: i16,
    pub unknown_1: i16,
    pub stick_lx: i16,
    pub stick_ly: i16,
    pub stick_lx_dont_use1: i16,
    pub stick_ly_dont_use1: i16,
    pub stick_rx: i16,
    pub stick_ry: i16,
    pub buttons_held_curr_frame: i32,
    pub buttons_tapped: i32,
    pub buttons_released: i32,
    pub buttons_held_prev_frame: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OnlineCTR {
    // 0x00
    pub current_state: i32,

    // 0x04
    pub page_number: i8, // to allow negative values
    pub count_press_x: u8,
    pub driver_count: u8,
    pub driver_id: u8,

    // 0x08
    pub locked_in_lap: u8,
    pub locked_in_level: u8,
    pub lap_id: u8,
    pub level_id: u8,

    // 0x0C
    pub is_booted_ps1: u8,
    pub locked_in_character: u8,
    pub locked_in_engine: u8,
    pub room_count: u8,
    pub drivers_ended_count: u8,

    // 0x10
    pub server_country: u8,
    pub server_room: u8,
    pub server_lock_in1: u8,
    pub server_lock_in2: u8,

    // 0x14
    pub planet_lev: u8,
    pub client_busy: u8,
    pub locked_in_special: u8,
    pub special: u8,
    pub warpclock: u8,
    pub finish_race_timer: u8,
    pub padding: i8,

    // 0x18
    pub client_count: [i8; 16],

    // 0x28 - determines if client and emulator are still connected
    pub windows_client_sync: i8,

    // 0x30
    pub locked_in_characters: [i8; MAX_NUM_PLAYERS],
    pub locked_in_engines: [i8; MAX_NUM_PLAYERS], // TODO; in the original this was called enginee, but that seems like a typo so I renamed it, idk if that will cause issues
    pub engine_type: [i8; MAX_NUM_PLAYERS],

    // 0x38
    pub name_buffer: [[u8; NAME_LENGTH + 1]; MAX_NUM_PLAYERS], // +1 for nullterm

    pub race_stats: [RaceStats; MAX_NUM_PLAYERS],

    pub psx_version: i32,
    pub pc_version: i32,
    pub server_version: i32,

    pub shoot: [ShootSlot; MAX_NUM_PLAYERS],

    pub frames_unsynced: i32,         // frames that the client didn't update
    pub last_windows_client_sync: i8, // last windowsClientSync counter
    pub ready_to_send: i32,           // when to start the client.exe loop
    pub auto_retry_join_room_index: i32, // queue to join
    pub gamemodes: [bool; 18],        // array of booleans for each gamemode
    pub room_type: u8,
    pub r_type_locked: u8,
    pub room_password_sequence: [u8; 8],
    pub password_entered: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShootSlot {
    pub juiced: u8,
    pub weapon: u8,
    pub flags: u8,
    pub now: u8,
}

pub struct CgEverythingKart {
    pub wumpa: u8,
    pub reserves: bool,
    pub kart_rot1: u8,
    pub kart_rot2: u8,
    pub button_hold: u8,
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
}
impl CgEverythingKart {
    pub fn to_bytes(&self) -> [u8; 10] {
        let b0 = (ClientMsg::RaceData as u8) & 0x0F
            | (self.wumpa & 0x07) << 4
            | (self.reserves as u8) << 7;
        [
            b0,
            self.kart_rot1,
            self.kart_rot2,
            self.button_hold,
            self.pos_x.to_le_bytes()[0],
            self.pos_x.to_le_bytes()[1],
            self.pos_y.to_le_bytes()[0],
            self.pos_y.to_le_bytes()[1],
            self.pos_z.to_le_bytes()[0],
            self.pos_z.to_le_bytes()[1],
        ]
    }
}
// CG_MessageWeapon — 2 bytes
pub struct CgMessageWeapon {
    pub juiced: bool,
    pub flags: u8,
    pub weapon: u8,
}
impl CgMessageWeapon {
    pub fn to_bytes(&self) -> [u8; 2] {
        let b0 = (ClientMsg::Weapon as u8) & 0x0F
            | (self.juiced as u8) << 4
            | 0 << 5  // padding
            | (self.flags & 0x03) << 6;
        let b1 = self.weapon & 0x0F;
        [b0, b1]
    }
}

pub struct SgEverythingKart {
    pub client_id: u8,
    pub wumpa: u8,
    pub bool_reserves: bool,
    pub kart_rot1: u8,
    pub kart_rot2: u8,
    pub button_hold: u8,
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
}
impl SgEverythingKart {
    pub fn from_bytes(data: &[u8]) -> Self {
        assert!(data.len() >= 10);
        SgEverythingKart {
            client_id: data[1] & 0x07,
            wumpa: (data[0] >> 4) & 0x07,
            bool_reserves: (data[0] >> 7) != 0,
            kart_rot1: (data[1] >> 3) & 0x1F,
            kart_rot2: data[2],
            button_hold: data[3],
            pos_x: i16::from_le_bytes([data[4], data[5]]),
            pos_y: i16::from_le_bytes([data[6], data[7]]),
            pos_z: i16::from_le_bytes([data[8], data[9]]),
        }
    }
}

// TODO: REVISAR LO DE OPENCODE, DEJE CODIGO GENERADO

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    LaunchEnterPid = 0,
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
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum GameMode {
    Normal = 0,
    Mirror,
    IcyTracks,
    Itemless,
    MoonMode,
    RetroFueled,
    FirstPerson,
    BossRace,
    DemoCamera,
    NVerted,
    Shortcutless,
    Night,
    Darkness,
    ItemChaos,
    Survival,
    SurvivalTimer,
    VanillaItems,
    WallDrive,
}
#[repr(u8)]
pub enum ServerMsg {
    // SG_
    Rooms = 0,
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
#[repr(u8)]
pub enum ClientMsg {
    // CG_
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

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

pub const DRIVER_COURSE_OFFSET: u32 = 0x514;
pub const DRIVER_BESTLAP_OFFSET: u32 = 0x63C;

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

// global.h:540-548 — 1 byte
pub struct CgHeader;
impl CgHeader {
    pub fn to_bytes() -> [u8; 1] {
        [(ClientMsg::StartRace as u8) & 0x0F]
    }
}

// global.h:550-557 — 2 bytes
pub struct CgMessageRoom {
    pub room: u8,
}
impl CgMessageRoom {
    pub fn to_bytes(&self) -> [u8; 2] {
        [(ClientMsg::JoinRoom as u8) & 0x0F, self.room]
    }
}

// global.h:510-515 — 3 bytes
pub struct CgMessageRoomType {
    pub room_type: u8,
    pub r_type_locked: u8,
}
impl CgMessageRoomType {
    pub fn to_bytes(&self) -> [u8; 3] {
        [(ClientMsg::RoomType as u8) & 0x0F, self.room_type, self.r_type_locked]
    }
}

// global.h:517-524 — 11 bytes
pub struct CgMessageRoomTypePassword {
    pub room_type: u8,
    pub r_type_locked: u8,
    pub seq: [u8; 8],
}
impl CgMessageRoomTypePassword {
    pub fn to_bytes(&self) -> [u8; 11] {
        let mut buf = [0u8; 11];
        buf[0] = (ClientMsg::RoomType as u8) & 0x0F;
        buf[1] = self.room_type;
        buf[2] = self.r_type_locked;
        buf[3..11].copy_from_slice(&self.seq);
        buf
    }
}

// global.h:527-532 — 9 bytes
pub struct CgMessagePassword {
    pub seq: [u8; 8],
}
impl CgMessagePassword {
    pub fn to_bytes(&self) -> [u8; 9] {
        let mut buf = [0u8; 9];
        buf[0] = (ClientMsg::Password as u8) & 0x0F;
        buf[1..9].copy_from_slice(&self.seq);
        buf
    }
}

// global.h:559-567 — 13 bytes
pub struct CgMessageName {
    pub name: [u8; 12],
}
impl CgMessageName {
    pub fn to_bytes(&self) -> [u8; 13] {
        let mut buf = [0u8; 13];
        buf[0] = (ClientMsg::Name as u8) & 0x0F;
        buf[1..13].copy_from_slice(&self.name);
        buf
    }
}

// global.h:570-577 — 3 bytes
pub struct CgMessageTrack {
    pub track_id: u8, // 5 bits
    pub lap_id: u8,   // 8 bits
}
impl CgMessageTrack {
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (ClientMsg::Track as u8) & 0x0F,
            (self.track_id & 0x1F) | ((self.lap_id & 0x07) << 5),
            (self.lap_id >> 3) & 0x1F,
        ]
    }
}

// GASMOX_CLIENT.cpp:1257-1263 — 19 bytes (custom, variable length)
pub struct CgMessageSpecial {
    pub gamemodes: [bool; 18],
}
impl CgMessageSpecial {
    pub fn to_bytes(&self) -> [u8; 19] {
        let mut buf = [0u8; 19];
        buf[0] = (ClientMsg::Special as u8) & 0x0F;
        for i in 0..18 {
            buf[1 + i] = self.gamemodes[i] as u8;
        }
        buf
    }
}

// global.h:587-599 — 2 bytes
// byte0: type(4) | characterID(4), byte1: boolLockedIn(1) | padding(7)
pub struct CgMessageCharacter {
    pub character_id: u8,  // 4 bits
    pub bool_locked_in: bool, // 1 bit
}
impl CgMessageCharacter {
    pub fn to_bytes(&self) -> [u8; 2] {
        [
            (ClientMsg::Character as u8) & 0x0F | (self.character_id & 0x0F) << 4,
            self.bool_locked_in as u8,
        ]
    }
}

// global.h:600-612 — 3 bytes
// byte0: type(4) | clientID=0(3), byte1: enginetype(4) | boolLockedIn(1) | padding(3)
pub struct CgMessageEngine {
    pub enginetype: u8,      // 4 bits
    pub bool_locked_in: bool, // 1 bit
}
impl CgMessageEngine {
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (ClientMsg::Engine as u8) & 0x0F,
            (self.enginetype & 0x0F) | ((self.bool_locked_in as u8) & 1) << 4,
            0,
        ]
    }
}

// global.h:656-661 — 1 byte
// byte0: type(4) | padding(2) | warpclock(2)
pub struct CgMessageWarpclock {
    pub warpclock: u8, // 2 bits
}
impl CgMessageWarpclock {
    pub fn to_bytes(&self) -> [u8; 1] {
        [((ClientMsg::Warpclock as u8) & 0x0F) | ((self.warpclock & 0x03) << 6)]
    }
}

// global.h:662-667 — 2 bytes
// byte0: type(4) | unused(4), byte1: finish_timer(6) | unused(2)
pub struct CgMessageFinishTimer {
    pub finish_timer: u8, // 6 bits
}
impl CgMessageFinishTimer {
    pub fn to_bytes(&self) -> [u8; 2] {
        [
            (ClientMsg::FinishTimer as u8) & 0x0F,
            self.finish_timer & 0x3F,
        ]
    }
}

// global.h:668-675 — 12 bytes
pub struct CgMessageEndRace {
    pub course_time: i32,
    pub lap_time: i32,
}
impl CgMessageEndRace {
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0] = (ClientMsg::EndRace as u8) & 0x0F;
        buf[4..8].copy_from_slice(&self.course_time.to_le_bytes());
        buf[8..12].copy_from_slice(&self.lap_time.to_le_bytes());
        buf
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

// global.h:278-286
pub struct SgHeader {
    pub msg_type: u8, // 4 bits
}
impl SgHeader {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgHeader { msg_type: data[0] & 0x0F }
    }
}

// global.h:320-331
pub struct SgMessageClientStatus {
    pub client_id: u8,   // 4 bits
    pub num_clients: u8, // 4 bits
}
impl SgMessageClientStatus {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageClientStatus {
            client_id: (data[0] >> 4) & 0x0F,
            num_clients: data[1] & 0x0F,
        }
    }
}

// global.h:270-275
pub struct SgMessageRoomType {
    pub room_type: u8,
    pub r_type_locked: u8,
}
impl SgMessageRoomType {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageRoomType {
            room_type: data[1],
            r_type_locked: data[2],
        }
    }
}

// global.h:288-317 — 12 bytes
// byte0: type(4)|padding(4), byte1: numRooms, bytes2-3: version(LE u16)
// bytes4-11: 8 bytes → 16 nibbles (clientCount[0..15])
pub struct SgMessageRooms {
    pub num_rooms: u8,
    pub version: u16,
    pub client_count: [i8; 16],
}
impl SgMessageRooms {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut client_count = [0i8; 16];
        for i in 0..8 {
            client_count[i * 2] = (data[4 + i] & 0x0F) as i8;
            client_count[i * 2 + 1] = (data[4 + i] >> 4) as i8;
        }
        SgMessageRooms {
            num_rooms: data[1],
            version: u16::from_le_bytes([data[2], data[3]]),
            client_count,
        }
    }
}

// global.h:333-337
pub struct SgMessagePasswordRejected;
impl SgMessagePasswordRejected {
    pub fn from_bytes(_data: &[u8]) -> Self {
        SgMessagePasswordRejected
    }
}

// global.h:340-352 — 14 bytes
pub struct SgMessageName {
    pub client_id: u8,       // 4 bits
    pub num_clients: u8,     // 4 bits
    pub name: [u8; 12],
}
impl SgMessageName {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut name = [0u8; 12];
        name.copy_from_slice(&data[2..14]);
        SgMessageName {
            client_id: data[1] & 0x0F,
            num_clients: data[1] >> 4,
            name,
        }
    }
}

// global.h:356-364 — 3 bytes
// byte0: type(4) | padding(4), byte1: trackID(5) | unused(3), byte2: lapID(8)
pub struct SgMessageTrack {
    pub track_id: u8, // 5 bits
    pub lap_id: u8,   // 8 bits
}
impl SgMessageTrack {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageTrack {
            track_id: data[1] & 0x1F,
            lap_id: data[2], // field doesn't fit in byte1 → entire byte2
        }
    }
}

// global.h:367-373 — 19 bytes (1 header + 18 booleans)
pub struct SgMessageSpecial {
    pub gamemodes: [bool; 18],
}
impl SgMessageSpecial {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut gamemodes = [false; 18];
        for i in 0..18 {
            gamemodes[i] = data[1 + i] != 0;
        }
        SgMessageSpecial { gamemodes }
    }
}

// global.h:376-390 — 2 bytes
// byte0: type(4) | clientID(3) | boolLockedIn(1), byte1: characterID(4) | padding(4)
pub struct SgMessageCharacter {
    pub client_id: u8,       // 3 bits
    pub bool_locked_in: bool, // 1 bit
    pub character_id: u8,    // 4 bits
}
impl SgMessageCharacter {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageCharacter {
            client_id: (data[0] >> 4) & 0x07,
            bool_locked_in: (data[0] >> 7) & 1 != 0,
            character_id: data[1] & 0x0F,
        }
    }
}

// global.h:391-403 — 3 bytes
// byte0: type(4) | clientID(3), byte1: enginetype(4) | boolLockedIn(1) | padding(3)
pub struct SgMessageEngine {
    pub client_id: u8,       // 3 bits
    pub enginetype: u8,      // 4 bits
    pub bool_locked_in: bool, // 1 bit
}
impl SgMessageEngine {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageEngine {
            client_id: (data[0] >> 4) & 0x07,
            enginetype: data[1] & 0x0F,
            bool_locked_in: (data[1] >> 4) & 1 != 0,
        }
    }
}

// global.h:436-450 — 2 bytes
// byte0: type(4) | clientID(3) | juiced(1)
// byte1: weapon(4) | flags(2) | padding(2)
pub struct SgMessageWeapon {
    pub client_id: u8,  // 3 bits
    pub juiced: bool,   // 1 bit
    pub weapon: u8,     // 4 bits
    pub flags: u8,      // 2 bits
}
impl SgMessageWeapon {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageWeapon {
            client_id: (data[0] >> 4) & 0x07,
            juiced: (data[0] >> 7) & 1 != 0,
            weapon: data[1] & 0x0F,
            flags: (data[1] >> 4) & 0x03,
        }
    }
}

// global.h:463-470 — 12 bytes
pub struct SgMessageEndRace {
    pub client_id: u8, // 4 bits
    pub course_time: i32,
    pub lap_time: i32,
}
impl SgMessageEndRace {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageEndRace {
            client_id: data[1] & 0x0F,
            course_time: i32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            lap_time: i32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        }
    }
}

// global.h:452-457 — 1 byte
// byte0: type(4) | padding(2) | warpclock(2)
pub struct SgMessageWarpclock {
    pub warpclock: u8, // 2 bits
}
impl SgMessageWarpclock {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageWarpclock { warpclock: (data[0] >> 6) & 0x03 }
    }
}

// global.h:458-462 — 2 bytes
// byte0: type(4) | unused(4), byte1: finish_timer(6) | unused(2)
pub struct SgMessageFinishTimer {
    pub finish_timer: u8, // 6 bits
}
impl SgMessageFinishTimer {
    pub fn from_bytes(data: &[u8]) -> Self {
        SgMessageFinishTimer { finish_timer: data[1] & 0x3F }
    }
}

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

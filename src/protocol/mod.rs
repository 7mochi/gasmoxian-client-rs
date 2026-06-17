/// Protocol definitions for the OnlineCTR protocol.
///
/// ## Wire format
///
/// Server → Client packets are framed with a 3-byte header:
/// ```text
///   msg_type: u8,  length: u16 (LE),  payload: [u8; length]
/// ```
/// The `ServerMessage` enum encodes the `msg_type` byte. Payload is
/// deserialized by the corresponding server-side struct.
///
/// Client → Server packets use ENet's reliable/unreliable channels directly.
/// The first byte (or low nibble) is always the `ClientMessage` type.
///
/// ## Shared memory
///
/// `OnlineCTR` is a `repr(C)` struct mmap'd from DuckStation's PS1 RAM
/// at `0x8000C000`. The client reads/writes it to communicate with the
/// game running inside the emulator.
use deku::{DekuRead, DekuWrite};
use num_derive::FromPrimitive;

pub mod client;
pub mod server;

/// Current protocol version. Must match between server, client, and PS1 binary.
pub const GASMOXIAN_VERSION: i8 = 3;

pub const MAX_NUM_PLAYERS_NORMAL: usize = 8;
pub const MAX_NUM_PLAYERS_TOURNAMENT: usize = 4;
pub const MAX_NUM_PLAYERS: usize = MAX_NUM_PLAYERS_NORMAL;

/// Maximum visible characters in a player name (excluding null terminator).
pub const MAX_NAME_LENGTH: usize = 11;

/// Per-player race stats stored in shared memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaceStats {
    /// Driver slot index
    pub slot: i32,
    /// Final race time in milliseconds
    pub final_time: i32,
    /// Best lap time in milliseconds
    pub best_lap: i32,
}

/// Per-player weapon/shoot slot stored in shared memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShootSlot {
    /// 1 = weapon is juiced (powered up)
    pub juiced: u8,
    /// Weapon item ID
    pub weapon: u8,
    /// Weapon behavior flags
    pub flags: u8,
    /// 1 = weapon being used right now
    pub now: u8,
}

/// Shared memory block mapped at `0x8000C000` in DuckStation PS1 RAM.
/// Max size: 0x400 bytes (1024).
///
/// The client (via `Ps1Memory`) reads/writes this struct to synchronize
/// game state with the emulated PS1 binary.
///
//  Memory map (offsets relative to base 0x8000C000):
//
//   Offset  Campo                 Tipo        Descripción
//   ──────  ────────────────────  ──────────  ──────────────────────────────────────
//   0x00    current_state         i32         Current ClientState enum value
//   0x04    page_number           i8          UI page index
//   0x05    count_press_x         u8          X button press counter
//   0x06    driver_count          u8          Number of connected drivers
//   0x07    driver_id             u8          This client's driver slot
//   0x08    locked_in_lap         u8          1 = lap selection locked in
//   0x09    locked_in_level       u8          1 = level selection locked in
//   0x0A    lap_id                u8          Selected lap count
//   0x0B    level_id              u8          Selected track/level ID
//   0x0C    is_booted_ps1         u8          1 = PS1 has been booted
//   0x0D    locked_in_character   u8          1 = character locked in
//   0x0E    locked_in_engine      u8          1 = engine locked in
//   0x0F    room_count            u8          Number of rooms from server
//   0x10    drivers_ended_count   u8          Number of drivers who finished
//   0x11    server_country        u8          Server country region index
//   0x12    server_room           u8          Current room index
//   0x13    server_lock_in1       u8          Server lock flag 1
//   0x14    server_lock_in2       u8          Server lock flag 2
//   0x15    planet_lev            u8          Planet level (special)
//   0x16    client_busy           u8          1 = client is busy (loading/racing)
//   0x17    locked_in_special     u8          1 = special/gamemode locked in
//   0x18    special               u8          Legacy special byte (replaced by gamemodes)
//   0x19    warpclock             u8          Warp clock state
//   0x1A    finish_race_timer     u8          Finish countdown timer
//   0x1B    padding               i8          Alignment padding
//   0x1C    client_count[16]      i8[16]      Player count per room (nibbles)
//   0x2C    windows_client_sync   i8          PC sync counter (incremented each frame)
//   0x30    locked_in_characters  i8[8]       Per-driver character lock state
//   0x38    locked_in_engines     i8[8]       Per-driver engine lock state
//   0x40    engine_type           i8[8]       Per-driver engine type
//   0x48    name_buffer           u8[8][12]   Per-driver name strings
//   0x98    race_stats            RaceStats[8] Per-driver race results
//   0xF8    psx_version           i32         PS1 binary version
//   0xFC    pc_version            i32         PC client version
//   0x100   server_version        i32         Server version
//   0x104   shoot                 ShootSlot[8] Per-driver weapon slots
//   0x124   frames_unsynced       i32         Frames without PC sync (disconnect timeout)
//   0x128   last_windows_client_sync i8       Last seen PC sync value
//   0x12C   ready_to_send         i32         Flag: data ready for network send
//   0x130   auto_retry_join_room  i32         Auto-retry room index (-1 = disabled)
//   0x134   gamemodes             bool[18]    Game mode toggle flags
//   0x146   room_type             u8          Room category
//   0x147   room_type_locked      u8          1 = room type locked
//   0x148   room_password_sequence u8[8]      Host-set password sequence
//   0x150   password_entered      u8[8]       Player-entered password attempt
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

/// Togglable game modes for the lobby.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Gamemode {
    Normal,
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

/// State machine values written to `OnlineCTR.current_state` by the PS1 binary.
/// The client reads this field each frame to dispatch the correct handler.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
pub enum ClientState {
    /// PID entry screen
    LaunchEnterPid,
    /// Server selection screen
    LaunchPickServer,
    /// Room selection screen
    LaunchPickRoom,
    /// Error/connection failed
    LaunchError,
    /// Password entry popup
    LaunchEnterPassword,
    /// Role assignment (host/guest)
    LobbyAssignRole,
    /// Host picks the track
    LobbyHostTrackPick,
    /// Host picks special/gamemodes
    LobbySpecialPick,
    /// Guest waits for host's track pick
    LobbyGuestTrackWait,
    /// Character selection
    LobbyCharacterPick,
    /// Engine selection
    LobbyEnginePick,
    /// Waiting for all players to load
    LobbyWaitForLoading,
    /// Starting to load the race
    LobbyStartLoading,
    /// Waiting for race to begin
    GameWaitForRace,
    /// Race is in progress
    GameStartRace,
    /// Race has ended
    GameEndRace,
}

/// Client → Server message types.
///
/// The low 4 bits of the first byte identify the message type.
/// Used with `DekuRead`/`DekuWrite` derive for serialization.
#[derive(Debug, Clone, Copy, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
#[repr(u8)]
pub enum ClientMessage {
    /// Join a room (payload: room index)
    JoinRoom = 0,
    /// Set room type
    RoomType,
    /// Send player name
    Name,
    /// Select track
    Track,
    /// Toggle game modes
    Special,
    /// Select character
    Character,
    /// Select engine
    Engine,
    /// Start race signal
    StartRace,
    /// RaceData / kart state (unreliable)
    RaceData,
    /// Weapon pickup/use
    Weapon,
    /// Warp clock state
    Warpclock,
    /// Finish timer sync
    FinishTimer,
    /// Race finished with times
    EndRace,
    /// Room password entry
    Password,
}

/// Server → Client message types.
///
/// Each packet on the wire starts with this byte, followed by a 2-byte
/// little-endian length, then the payload.
#[repr(u8)]
#[derive(FromPrimitive, Debug)]
pub enum ServerMessage {
    /// Room list (12 bytes)
    Rooms,
    /// Room type assignment (3 bytes)
    RoomType,
    /// Room type rejected (signal)
    RoomTypeRejected,
    /// New client joined (2 bytes)
    NewClient,
    /// Player name broadcast (14 bytes)
    Name,
    /// Track pick (3 bytes)
    Track,
    /// Gamemode update (19 bytes)
    Special,
    /// Character pick (2 bytes)
    Character,
    /// Engine pick (3 bytes)
    Engine,
    /// Start loading signal
    StartLoading,
    /// Start race signal
    StartRace,
    /// RaceData / kart state (10 bytes, unreliable)
    RaceData,
    /// Weapon pickup (2 bytes)
    Weapon,
    /// Warp clock update (1 byte)
    Warpclock,
    /// Finish timer update (2 bytes)
    FinishTimer,
    /// Race finished (12 bytes)
    EndRace,
    /// Server is shutting down
    ServerClosed,
    /// Password was rejected
    PasswordRejected,
}

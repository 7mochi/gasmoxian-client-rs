//! Pure commands returned by state functions instead of performing I/O.
//! Every state and packet handler returns `Vec<Effect>`. The caller
//! ([`io::exec_effects`]) drains the vector and applies each variant
//! to the real system (shared memory, enet socket, terminal). This
//! keeps all game logic deterministic and I/O-free.

use crate::protocol::{ClientState, MAX_NAME_LENGTH, RaceStats, ShootSlot};

#[derive(Debug, PartialEq, Clone)]
pub enum Effect {
    /// Send a reliable packet to the server
    SendReliable(Vec<u8>),
    /// Send an unsequenced (unreliable) packet
    SendUnsequenced(Vec<u8>),
    /// Send an enet ping to keep the connection alive
    Ping,

    /// Change the state machine's current state
    SetState(ClientState),
    /// Set state to a raw i32 value (for values outside the enum, e.g. -1)
    SetStateRaw(i32),
    /// Mark client as busy (0 = free, 1 = busy)
    SetClientBusy(u8),
    /// Auto-retry room index (-1 = disabled)
    SetAutoRetryRoomIndex(i32),
    /// Set driver_id (this player's slot in the room)
    SetDriverId(u8),
    /// Value for ready_to_send (PS1 sync)
    SetReadyToSend(i32),
    /// Value for windows_client_sync (frame counter)
    SetWindowsClientSync(i8),
    /// server_lock_in1
    SetServerLockIn1(u8),
    /// server_lock_in2
    SetServerLockIn2(u8),
    /// room_type
    SetRoomType(u8),
    /// room_type_locked
    SetRoomTypeLocked(u8),
    /// finish_race_timer (visual countdown)
    SetFinishRaceTimer(u8),
    /// drivers_ended_count (racers who finished)
    SetDriversEndedCount(u8),
    /// Write the `now` field of a shoot slot
    SetShootNow { slot: usize, value: u8 },
    /// Write a race stats entry
    WriteRaceStats { slot: usize, stats: RaceStats },

    /// Write a byte at an absolute PS1 address
    WriteU8(u32, u8),
    /// Write a 16-bit word at an absolute PS1 address
    WriteU16(u32, u16),
    /// Write a 32-bit dword at an absolute PS1 address
    WriteU32(u32, u32),
    /// Write a byte buffer at an absolute PS1 address
    WriteBytes { addr: u32, data: Vec<u8> },

    /// level_id (selected track)
    SetLevelId(u8),
    /// lap_id (selected laps)
    SetLapId(u8),
    /// locked_in_lap
    SetLockedInLap(u8),
    /// locked_in_level
    SetLockedInLevel(u8),
    /// locked_in_special
    SetLockedInSpecial(u8),
    /// locked_in_character (single byte)
    SetLockedInCharacterByte(u8),
    /// locked_in_engine (single byte)
    SetLockedInEngineByte(u8),
    /// special (legacy byte)
    SetSpecial(u8),
    /// server_version from the server
    SetServerVersion(i32),
    /// pc_version from the server
    SetPcVersion(i32),
    /// room_count from the server
    SetRoomCount(u8),
    /// Set the entire client_count array
    SetClientCount([i8; 16]),
    /// driver_count (number of connected players)
    SetDriverCount(u8),
    /// Set one slot of name_buffer
    SetNameBuffer {
        slot: usize,
        data: [u8; MAX_NAME_LENGTH + 1],
    },
    /// Set a specific gamemode by index
    SetGamemode { index: usize, value: bool },
    /// locked_in_characters[slot]
    SetLockedInCharacter { slot: usize, value: i8 },
    /// locked_in_engines[slot]
    SetLockedInEngine { slot: usize, value: i8 },
    /// engine_type[slot]
    SetEngineType { slot: usize, value: i8 },
    /// warpclock
    SetWarpclock(u8),
    /// password_entered (full reset to 0)
    SetPasswordEntered([u8; 8]),
    /// room_password_sequence (full reset to 0)
    SetRoomPasswordSequence([u8; 8]),
    /// Write a full ShootSlot
    SetShoot { slot: usize, shoot: ShootSlot },

    /// Force an immediate enet disconnect
    DisconnectNow,

    /// Debug message (only shown with LOG=debug)
    LogDebug(String),
    /// Informational message to the user
    LogInfo(String),
    /// Success message
    LogOk(String),
    /// Error message
    LogErr(String),
}

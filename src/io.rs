//! Sole I/O boundary of the sans-IO architecture.
//!
//! [`exec_effects`] drains a `Vec<Effect>` and applies each variant
//! to either PS1 shared memory, the enet connection, or the terminal.
//! There is no other place in the program that performs observable
//! side effects on external systems.

use crate::{console, effect::Effect, enet::EnetClient, ps1_memory::Ps1Memory};

/// Executes all accumulated effects, draining the vector.
/// This is the only place in the program that performs real I/O.
pub fn exec_effects(
    effects: &mut Vec<Effect>,
    ps1_memory: &mut Ps1Memory,
    net: &mut Option<EnetClient>,
) {
    // Pre-scan for critical fields that the PS1 reads during frame processing.
    //
    // DuckStation's PS1 emulation reads shared memory continuously, even while
    // `exec_effects` is still writing. Without this pre-scan, `SetRoomType` from
    // the RoomType message (processed first) would be written to shared memory
    // before `SetDriverId` and `SetState` from NewClient (processed later).
    // If the PS1 reads at that instant, it sees `driver_id=0, locked=0` and
    // enters host mode instead of guest.
    //
    // Writing these three fields together first eliminates the window where
    // state and driver_id are inconsistent with each other.
    for effect in effects.iter() {
        match effect {
            Effect::SetDriverId(v) => ps1_memory.online_ctr_mut().driver_id = *v,
            Effect::SetRoomTypeLocked(v) => ps1_memory.online_ctr_mut().room_type_locked = *v,
            Effect::SetState(s) => ps1_memory.online_ctr_mut().current_state = *s as i32,
            Effect::SetStateRaw(v) => ps1_memory.online_ctr_mut().current_state = *v,
            _ => {}
        }
    }

    for effect in effects.drain(..) {
        match effect {
            Effect::SetState(s) => {
                ps1_memory.online_ctr_mut().current_state = s as i32;
            }
            Effect::SetStateRaw(v) => {
                ps1_memory.online_ctr_mut().current_state = v;
            }
            Effect::SetClientBusy(v) => {
                ps1_memory.online_ctr_mut().client_busy = v;
            }
            Effect::SetFinishRaceTimer(v) => {
                ps1_memory.online_ctr_mut().finish_race_timer = v;
            }
            Effect::SetRoomType(v) => {
                ps1_memory.online_ctr_mut().room_type = v;
            }
            Effect::SetRoomTypeLocked(v) => {
                ps1_memory.online_ctr_mut().room_type_locked = v;
            }
            Effect::SendReliable(bytes) => {
                if let Some(net) = net
                    && let Err(e) = net.send_reliable(&bytes)
                {
                    console::err(format!("send_reliable failed: {e}"));
                }
            }
            Effect::SendUnsequenced(bytes) => {
                if let Some(net) = net {
                    let _ = net.send_unsequenced(&bytes);
                }
            }
            Effect::Ping => {
                if let Some(net) = net {
                    net.ping();
                }
            }
            Effect::DisconnectNow => {
                if let Some(net) = net {
                    net.disconnect_now();
                }
            }
            Effect::WriteU8(addr, val) => {
                if let Err(e) = ps1_memory.write_u8(addr, val) {
                    console::err(format!("write_u8(0x{addr:08X}) failed: {e}"));
                }
            }
            Effect::WriteU16(addr, val) => {
                if let Err(e) = ps1_memory.write_u16(addr, val) {
                    console::err(format!("write_u16(0x{addr:08X}) failed: {e}"));
                }
            }
            Effect::WriteU32(addr, val) => {
                if let Err(e) = ps1_memory.write_u32(addr, val) {
                    console::err(format!("write_u32(0x{addr:08X}) failed: {e}"));
                }
            }
            Effect::WriteBytes { addr, data } => {
                for (i, &b) in data.iter().enumerate() {
                    if let Err(e) = ps1_memory.write_u8(addr.wrapping_add(i as u32), b) {
                        console::err(format!("write_bytes(0x{addr:08X}+{i}) failed: {e}"));
                        break;
                    }
                }
            }
            Effect::SetShootNow { slot, value } => {
                if slot < ps1_memory.online_ctr().shoot.len() {
                    ps1_memory.online_ctr_mut().shoot[slot].now = value;
                }
            }
            Effect::WriteRaceStats { slot, stats } => {
                if slot < ps1_memory.online_ctr().race_stats.len() {
                    ps1_memory.online_ctr_mut().race_stats[slot] = stats;
                }
            }
            Effect::SetDriversEndedCount(v) => {
                ps1_memory.online_ctr_mut().drivers_ended_count = v;
            }
            Effect::SetLevelId(v) => {
                ps1_memory.online_ctr_mut().level_id = v;
            }
            Effect::SetLapId(v) => {
                ps1_memory.online_ctr_mut().lap_id = v;
            }
            Effect::SetLockedInLap(v) => {
                ps1_memory.online_ctr_mut().locked_in_lap = v;
            }
            Effect::SetLockedInLevel(v) => {
                ps1_memory.online_ctr_mut().locked_in_level = v;
            }
            Effect::SetLockedInSpecial(v) => {
                ps1_memory.online_ctr_mut().locked_in_special = v;
            }
            Effect::SetLockedInCharacterByte(v) => {
                ps1_memory.online_ctr_mut().locked_in_character = v;
            }
            Effect::SetLockedInEngineByte(v) => {
                ps1_memory.online_ctr_mut().locked_in_engine = v;
            }
            Effect::SetSpecial(v) => {
                ps1_memory.online_ctr_mut().special = v;
            }
            Effect::SetServerVersion(v) => {
                ps1_memory.online_ctr_mut().server_version = v;
            }
            Effect::SetPcVersion(v) => {
                ps1_memory.online_ctr_mut().pc_version = v;
            }
            Effect::SetRoomCount(v) => {
                ps1_memory.online_ctr_mut().room_count = v;
            }
            Effect::SetClientCount(arr) => {
                ps1_memory.online_ctr_mut().client_count = arr;
            }
            Effect::SetDriverCount(v) => {
                ps1_memory.online_ctr_mut().driver_count = v;
            }
            Effect::SetNameBuffer { slot, data } => {
                if slot < ps1_memory.online_ctr().name_buffer.len() {
                    ps1_memory.online_ctr_mut().name_buffer[slot] = data;
                }
            }
            Effect::SetGamemode { index, value } => {
                if index < ps1_memory.online_ctr().gamemodes.len() {
                    ps1_memory.online_ctr_mut().gamemodes[index] = value;
                }
            }
            Effect::SetLockedInCharacter { slot, value } => {
                if slot < ps1_memory.online_ctr().locked_in_characters.len() {
                    ps1_memory.online_ctr_mut().locked_in_characters[slot] = value;
                }
            }
            Effect::SetLockedInEngine { slot, value } => {
                if slot < ps1_memory.online_ctr().locked_in_engines.len() {
                    ps1_memory.online_ctr_mut().locked_in_engines[slot] = value;
                }
            }
            Effect::SetEngineType { slot, value } => {
                if slot < ps1_memory.online_ctr().engine_type.len() {
                    ps1_memory.online_ctr_mut().engine_type[slot] = value;
                }
            }
            Effect::SetWarpclock(v) => {
                ps1_memory.online_ctr_mut().warpclock = v;
            }
            Effect::SetPasswordEntered(arr) => {
                ps1_memory.online_ctr_mut().password_entered = arr;
            }
            Effect::SetRoomPasswordSequence(arr) => {
                ps1_memory.online_ctr_mut().room_password_sequence = arr;
            }
            Effect::SetShoot { slot, shoot } => {
                if slot < ps1_memory.online_ctr().shoot.len() {
                    ps1_memory.online_ctr_mut().shoot[slot] = shoot;
                }
            }
            Effect::SetAutoRetryRoomIndex(v) => {
                ps1_memory.online_ctr_mut().auto_retry_join_room_index = v;
            }
            Effect::LogDebug(msg) => console::debug(msg),
            Effect::LogInfo(msg) => console::info(msg),
            Effect::LogOk(msg) => console::ok(msg),
            Effect::LogErr(msg) => console::err(msg),
            _ => {}
        }
    }
}

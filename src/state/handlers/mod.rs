use deku::DekuContainerRead;
use num_traits::FromPrimitive;

use crate::{
    effect::Effect,
    protocol::{
        ServerMessage::{self},
        server::{
            Character, ClientStatus, EndRace, Engine, FinishTimer, Kart, Name, RoomType, Rooms,
            Special, Track, WarpClock, Weapon,
        },
    },
    ps1_snapshot::OnlineCtrSnapshot,
    state::GameState,
};

pub mod character;
pub mod end_race;
pub mod engine;
pub mod finish_timer;
pub mod name;
pub mod new_client;
pub mod password_rejected;
pub mod race_data;
pub mod room_type;
pub mod rooms;
pub mod special;
pub mod start_loading;
pub mod start_race;
pub mod track;
pub mod warp_clock;
pub mod weapon;

pub fn process_receive_event(
    ctr: &OnlineCtrSnapshot,
    state: &mut GameState,
    data: &[u8],
) -> Vec<Effect> {
    let msg_type = match ServerMessage::from_u8(data[0] & 0x0F) {
        Some(t) => t,
        None => {
            return vec![Effect::LogDebug(format!(
                "unhandled message type: {}",
                data[0] & 0x0F
            ))];
        }
    };

    match msg_type {
        ServerMessage::Rooms => match Rooms::from_bytes((data, 0)) {
            Ok((_, msg)) => rooms::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Rooms: {e:?}"
            ))],
        },
        ServerMessage::RoomType => match RoomType::from_bytes((data, 0)) {
            Ok((_, msg)) => room_type::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize RoomType: {e:?}"
            ))],
        },
        ServerMessage::PasswordRejected => password_rejected::handle(state),
        ServerMessage::NewClient => match ClientStatus::from_bytes((data, 0)) {
            Ok((_, msg)) => new_client::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize ClientStatus: {e:?}"
            ))],
        },
        ServerMessage::Name => match Name::from_bytes((data, 0)) {
            Ok((_, msg)) => name::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Name: {e:?}"
            ))],
        },
        ServerMessage::Track => match Track::from_bytes((data, 0)) {
            Ok((_, msg)) => track::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Track: {e:?}"
            ))],
        },
        ServerMessage::Special => match Special::from_bytes((data, 0)) {
            Ok((_, msg)) => special::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Special: {e:?}"
            ))],
        },
        ServerMessage::Character => match Character::from_bytes((data, 0)) {
            Ok((_, msg)) => character::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Character: {e:?}"
            ))],
        },
        ServerMessage::Engine => match Engine::from_bytes((data, 0)) {
            Ok((_, msg)) => engine::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Engine: {e:?}"
            ))],
        },
        ServerMessage::StartLoading => start_loading::handle(),
        ServerMessage::StartRace => start_race::handle(),
        ServerMessage::RaceData => match Kart::from_bytes((data, 0)) {
            Ok((_, msg)) => race_data::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Kart: {e:?}"
            ))],
        },
        ServerMessage::Weapon => match Weapon::from_bytes((data, 0)) {
            Ok((_, msg)) => weapon::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize Weapon: {e:?}"
            ))],
        },
        ServerMessage::Warpclock => match WarpClock::from_bytes((data, 0)) {
            Ok((_, msg)) => warp_clock::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize WarpClock: {e:?}"
            ))],
        },
        ServerMessage::FinishTimer => match FinishTimer::from_bytes((data, 0)) {
            Ok((_, msg)) => finish_timer::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize FinishTimer: {e:?}"
            ))],
        },
        ServerMessage::EndRace => match EndRace::from_bytes((data, 0)) {
            Ok((_, msg)) => end_race::handle(ctr, state, msg),
            Err(e) => vec![Effect::LogDebug(format!(
                "failed to deserialize EndRace: {e:?}"
            ))],
        },
        _ => {
            vec![Effect::LogDebug(format!(
                "unhandled message type: {:?}",
                msg_type
            ))]
        }
    }
}

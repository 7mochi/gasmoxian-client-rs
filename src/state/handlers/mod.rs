use deku::DekuContainerRead;
use num_traits::FromPrimitive;

use crate::{
    console,
    enet::EnetClient,
    protocol::{
        ServerMessage::{self},
        server::{
            Character, ClientStatus, EndRace, Engine, FinishTimer, Kart, Name, RoomType, Rooms,
            Special, Track, WarpClock, Weapon,
        },
    },
    ps1_memory::Ps1Memory,
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

macro_rules! try_msg {
    ($ty:ty, $handler:expr, $data:expr) => {
        match <$ty>::from_bytes(($data, 0)) {
            Ok((_, msg)) => {
                if let Err(e) = $handler(msg) {
                    console::err(format!("{}: {:?}", stringify!($handler), e));
                }
            }
            Err(e) => console::debug(format!(
                "failed to deserialize {}: {:?}",
                stringify!($ty),
                e
            )),
        }
    };
}

macro_rules! try_handler {
    ($handler:expr) => {
        if let Err(e) = $handler {
            console::err(format!("{}: {:?}", stringify!($handler), e));
        }
    };
}

pub fn process_receive_event(
    ps1_memory: &mut Ps1Memory,
    net: &mut EnetClient,
    state: &mut GameState,
    data: &[u8],
) {
    let msg_type = ServerMessage::from_u8(data[0] & 0x0F).expect("invalid message type");

    match msg_type {
        ServerMessage::Rooms => {
            try_msg!(Rooms, |msg| rooms::handle(ps1_memory, state, msg), data);
        }
        ServerMessage::RoomType => {
            try_msg!(
                RoomType,
                |msg| room_type::handle(ps1_memory, state, msg),
                data
            );
        }
        ServerMessage::PasswordRejected => {
            try_handler!(password_rejected::handle(ps1_memory, net, state));
        }
        ServerMessage::NewClient => {
            try_msg!(
                ClientStatus,
                |msg| new_client::handle(ps1_memory, net, state, msg),
                data
            );
        }
        ServerMessage::Name => {
            try_msg!(Name, |msg| name::handle(ps1_memory, msg), data);
        }
        ServerMessage::Track => {
            try_msg!(Track, |msg| track::handle(ps1_memory, state, msg), data);
        }
        ServerMessage::Special => {
            try_msg!(Special, |msg| special::handle(ps1_memory, msg), data);
        }
        ServerMessage::Character => {
            try_msg!(Character, |msg| character::handle(ps1_memory, msg), data);
        }
        ServerMessage::Engine => {
            try_msg!(Engine, |msg| engine::handle(ps1_memory, msg), data);
        }
        ServerMessage::StartLoading => {
            try_handler!(start_loading::handle(ps1_memory));
        }
        ServerMessage::StartRace => {
            try_handler!(start_race::handle(ps1_memory));
        }
        ServerMessage::RaceData => {
            try_msg!(Kart, |msg| race_data::handle(ps1_memory, state, msg), data);
        }
        ServerMessage::Weapon => {
            try_msg!(Weapon, |msg| weapon::handle(ps1_memory, msg), data);
        }
        ServerMessage::Warpclock => {
            try_msg!(
                WarpClock,
                |msg| warp_clock::handle(ps1_memory, state, msg),
                data
            );
        }
        ServerMessage::FinishTimer => {
            try_msg!(
                FinishTimer,
                |msg| finish_timer::handle(ps1_memory, state, msg),
                data
            );
        }
        ServerMessage::EndRace => {
            try_msg!(
                EndRace,
                |msg| end_race::handle(ps1_memory, state, msg),
                data
            );
        }
        _ => {
            console::debug(format!("unhandled message type: {:?}", msg_type));
        }
    }
}

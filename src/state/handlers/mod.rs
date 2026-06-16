use deku::DekuContainerRead;
use num_traits::FromPrimitive;

use crate::{
    console,
    enet::EnetClient,
    protocol::{
        ServerMessage,
        server::{ClientStatus, Name, RoomType, Rooms},
    },
    ps1_memory::Ps1Memory,
    state::GameState,
};

pub mod name;
pub mod new_client;
pub mod password_rejected;
pub mod room_type;
pub mod rooms;

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
            try_msg!(RoomType, |msg| room_type::handle(ps1_memory, state, msg), data);
        }
        ServerMessage::PasswordRejected => {
            try_handler!(password_rejected::handle(ps1_memory, net, state));
        }
        ServerMessage::NewClient => {
            try_msg!(ClientStatus, |msg| new_client::handle(ps1_memory, net, state, msg), data);
        }
        ServerMessage::Name => {
            try_msg!(Name, |msg| name::handle(ps1_memory, msg), data);
        }
        _ => {}
    }
}

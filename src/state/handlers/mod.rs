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

pub fn process_receive_event(
    ps1_memory: &mut Ps1Memory,
    net: &mut EnetClient,
    state: &mut GameState,
    data: &[u8],
) {
    let msg_type = ServerMessage::from_u8(data[0] & 0x0F).expect("invalid message type");
    console::debug(format!(
        "Received message type: [{:?}] {:?} ({} bytes)",
        data,
        msg_type,
        data.len()
    ));

    match msg_type {
        ServerMessage::Rooms => {
            if let Ok((_, message)) = Rooms::from_bytes((data, 0)) {
                if let Err(e) = rooms::handle(ps1_memory, state, message) {
                    console::err(format!("Failed to handle rooms message: {:?}", e));
                }
            }
        }
        ServerMessage::RoomType => {
            if let Ok((_, message)) = RoomType::from_bytes((data, 0)) {
                if let Err(e) = room_type::handle(ps1_memory, state, message) {
                    console::err(format!("Failed to handle room type message: {:?}", e));
                }
            }
        }
        ServerMessage::PasswordRejected => {
            if let Err(e) = password_rejected::handle(ps1_memory, net, state) {
                console::err(format!(
                    "Failed to handle password rejected message: {:?}",
                    e
                ));
            }
        }
        ServerMessage::NewClient => {
            if let Ok((_, message)) = ClientStatus::from_bytes((data, 0)) {
                if let Err(e) = new_client::handle(ps1_memory, net, state, message) {
                    console::err(format!("Failed to handle new client message: {:?}", e));
                }
            }
        }
        ServerMessage::Name => {
            if let Ok((_, message)) = Name::from_bytes((data, 0)) {
                if let Err(e) = name::handle(ps1_memory, message) {
                    console::err(format!("Failed to handle name message: {:?}", e));
                }
            }
        }
        _ => {}
    }
}

use gasmoxian_client_rs_v2::{
    console,
    enet::EnetClient,
    filter::{self, DEFAULT_USERNAME},
    protocol::{ClientState, MAX_NAME_LENGTH},
    ps1_memory::Ps1Memory,
    state::{self, GameState},
};
use num_traits::FromPrimitive;

fn main() -> anyhow::Result<()> {
    console::print_banner();

    console::init_debug();
    console::debug("Debug mode enabled");

    let args: Vec<String> = std::env::args().collect();
    let mut username = if args.len() > 1 {
        args[1].clone()
    } else {
        console::prompt_username()
    };

    if filter::contains_prohibited_name(&username) {
        console::err("your username contains banned words, using default instead: \"gasmoxian\"");
        username = DEFAULT_USERNAME.to_string();
    }

    // Wait for DuckStation shared memory
    console::info("Waiting for the Gasmoxian binary to load...");

    let mut ps1_memory = loop {
        match Ps1Memory::connect() {
            Ok(mem) => break mem,
            Err(e) => {
                console::err(format!("{}", e));
                console::info("Retrying in 5 seconds...");

                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    };

    console::info("DuckStation shared memory found, starting client...");

    let mut net: Option<EnetClient> = None;
    let mut gamestate = GameState::new();

    // Copy the username into the gamestate, truncating it if it's too long
    gamestate.lobby.username = username.chars().take(MAX_NAME_LENGTH).collect();
    console::debug(format!(
        "Username set to \"{}\" ({} chars)",
        gamestate.lobby.username,
        gamestate.lobby.username.len()
    ));

    let name_bytes = gamestate.lobby.username.as_bytes();
    let len = name_bytes
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(name_bytes.len());

    let online_ctr = ps1_memory.online_ctr_mut();
    online_ctr.current_state = ClientState::LaunchEnterPid as i32;
    online_ctr.name_buffer[0].fill(0);
    online_ctr.name_buffer[0][..len].copy_from_slice(&name_bytes[..len]);
    online_ctr.auto_retry_join_room_index = -1;

    let mut prev_state_idx = ps1_memory.online_ctr().current_state;

    // main loop
    loop {
        // increment the sync counter to notify the PS1 game that the PC client
        // is alive and actively running
        let next_sync = ps1_memory.online_ctr().windows_client_sync.wrapping_add(1);
        ps1_memory.online_ctr_mut().windows_client_sync = next_sync;

        let state_idx = ps1_memory.online_ctr().current_state;

        if state_idx != prev_state_idx {
            console::debug(format!(
                "State transition: {} -> {}",
                prev_state_idx, state_idx
            ));
            prev_state_idx = state_idx;
        }

        // afk timer (only in character/engine selection)
        if state_idx >= ClientState::LobbyCharacterPick as i32
            && state_idx < ClientState::LobbyWaitForLoading as i32
        {
            state::afk_timer(&mut ps1_memory, net.as_mut(), &mut gamestate);
        }

        // disconnect check
        if state_idx >= ClientState::LaunchPickRoom as i32 {
            state::disconnect(&mut ps1_memory, net.as_mut(), &mut gamestate);
        }

        if let Some(current_state) = ClientState::from_i32(state_idx) {
            match current_state {
                ClientState::LaunchPickRoom => {
                    if let Some(addr) = gamestate.connection.server_addr.take() {
                        console::debug(format!("Attempting connection to {}", addr));
                        match EnetClient::new(addr) {
                            Ok(client) => {
                                console::ok("Successfully connected!");
                                net = Some(client);

                                let online_ctr = ps1_memory.online_ctr_mut();
                                online_ctr.driver_id = 0xFF;
                                online_ctr.current_state = ClientState::LaunchPickRoom as i32;
                                continue;
                            }
                            Err(e) => {
                                console::err(format!("Failed to connect! ({})", e));

                                ps1_memory.online_ctr_mut().current_state =
                                    ClientState::LaunchPickServer as i32;
                                continue;
                            }
                        }
                    }
                }

                ClientState::LaunchEnterPid => {
                    state::launch_enter_pid(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }

                ClientState::LaunchPickServer => {
                    state::launch_pick_server(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }

                _ => {}
            }
        }

        // process network messages
        state::process_network_messages(&mut ps1_memory, net.as_mut(), &mut gamestate);

        // frame sync
        state::frame_stall(&mut ps1_memory);
    }
}

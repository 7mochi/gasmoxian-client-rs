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
                    state::launch_pick_room(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::LaunchEnterPid => {
                    state::launch_enter_pid(&mut ps1_memory);
                }
                ClientState::LaunchPickServer => {
                    state::launch_pick_server(&mut ps1_memory, &mut gamestate);

                    if let Some(addr) = gamestate.connection.server_addr.take() {
                        console::debug(format!("Attempting connection to {}", addr));

                        let mut connected = false;
                        for attempt in 1..=3 {
                            match EnetClient::connect_with_handshake(addr) {
                                Ok(client) => {
                                    console::ok("Successfully connected!");
                                    net = Some(client);
                                    ps1_memory.online_ctr_mut().driver_id = 0xFF;
                                    ps1_memory.online_ctr_mut().client_busy = 0;
                                    ps1_memory.online_ctr_mut().current_state =
                                        ClientState::LaunchPickRoom as i32;
                                    connected = true;
                                    break;
                                }
                                Err(_) => {
                                    console::err(format!(
                                        "Failed to connect! Attempt {}/3...",
                                        attempt
                                    ));
                                }
                            }
                        }

                        if !connected {
                            ps1_memory.online_ctr_mut().server_lock_in1 = 0;
                            ps1_memory.online_ctr_mut().current_state =
                                ClientState::LaunchPickServer as i32;
                            ps1_memory.online_ctr_mut().client_busy = 0;
                            console::err("Returning to server selection.");
                        }
                    }
                }
                ClientState::LaunchError => {
                    state::launch_error(&mut ps1_memory);
                }
                ClientState::LaunchEnterPassword => {
                    state::launch_enter_password(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::LobbyAssignRole => {
                    state::lobby_assign_role(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::LobbyHostTrackPick => {
                    state::lobby_host_track_pick(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::LobbySpecialPick => {
                    state::lobby_special_pick(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::LobbyCharacterPick => {
                    state::lobby_character_pick(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::LobbyEnginePick => {
                    state::lobby_engine_pick(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::LobbyGuestTrackWait => {
                    state::lobby_guest_track_wait(&mut gamestate);
                }
                ClientState::LobbyWaitForLoading => {
                    state::lobby_wait_for_loading();
                }
                ClientState::LobbyStartLoading => {
                    state::lobby_start_loading(&mut ps1_memory, &mut gamestate);
                }
                ClientState::GameWaitForRace => {
                    state::game_wait_for_race(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::GameStartRace => {
                    state::game_start_race(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
                ClientState::GameEndRace => {
                    state::game_end_race(&mut ps1_memory, net.as_mut(), &mut gamestate);
                }
            }
        }

        // process network messages
        state::process_network_messages(&mut ps1_memory, net.as_mut(), &mut gamestate);

        // frame sync
        state::frame_stall(&mut ps1_memory);
    }
}

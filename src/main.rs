// GASMOX_CLIENT.cpp:1788-2064
use std::io::Write;

use gasmoxian_client_rs::{
    enet::EnetClient,
    filter::contains_prohibited_name,
    protocol::{ClientState, NAME_LENGTH},
    ps1mem::Ps1Mem,
    state::{frame_stall, process_new_messages, discon_select, afk_timer, GameState, STATE_FUNCTIONS},
};

fn main() {
    // Parse username
    let args: Vec<String> = std::env::args().collect();
    let mut name = [0u8; NAME_LENGTH + 1];

    if args.len() > 1 {
        let arg_bytes = args[1].as_bytes();
        let len = arg_bytes.len().min(NAME_LENGTH);
        name[..len].copy_from_slice(&arg_bytes[..len]);
    } else {
        print_banner(false);
        print!("Enter Your Username (11) = ");
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim();
        let bytes = input.as_bytes();
        let len = bytes.len().min(NAME_LENGTH);
        name[..len].copy_from_slice(&bytes[..len]);
    }

    if contains_prohibited_name(&name) {
        println!("\nyour username contains banned words, using default instead: \"gasmoxian\"");
        let default = b"gasmoxian\0";
        let len = default.len().min(NAME_LENGTH);
        name[..len].copy_from_slice(&default[..len]);
    }
    name[NAME_LENGTH] = 0;

    // Wait for DuckStation shared memory
    print_banner(true);
    println!();
    println!("Client: Waiting for the Gasmoxian binary to load...");

    let ps1 = loop {
        match Ps1Mem::connect() {
            Ok(mem) => break mem,
            Err(e) => {
                println!("{}", e);
                println!("Retrying in 5 seconds...");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    };

    let mut net: Option<EnetClient> = None;
    let mut state = GameState::new();

    // Copy name to GameState
    for i in 0..NAME_LENGTH {
        state.name[i] = name[i];
    }
    state.name[NAME_LENGTH] = 0;

    // Copy name to PS1 OnlineCTR
    let len = name.iter().position(|&c| c == 0).unwrap_or(NAME_LENGTH);
    let octr = ps1.octr_mut();
    octr.current_state = ClientState::LaunchEnterPid as i32;
    octr.name_buffer[0][..len].copy_from_slice(&name[..len]);
    octr.name_buffer[0][NAME_LENGTH] = 0;
    octr.auto_retry_join_room_index = -1;

    // Main loop
    loop {
        ps1.octr_mut().windows_client_sync = ps1.octr().windows_client_sync.wrapping_add(1);

        let state_idx = ps1.octr().current_state;

        // AFK timer (only in character/engine selection)
        if state_idx >= ClientState::LobbyCharacterPick as i32
            && state_idx < ClientState::LobbyWaitForLoading as i32
        {
            if let Some(ref mut n) = net {
                afk_timer(&ps1, n, &mut state);
            }
        }

        // Disconnect check
        if state_idx >= ClientState::LaunchPickRoom as i32 {
            if let Some(ref mut n) = net {
                discon_select(&ps1, n, &mut state);
            }
        }

        // State machine dispatch
        if state_idx >= 0 {
            let idx = state_idx as usize;
            if idx < STATE_FUNCTIONS.len() {
                // Create EnetClient if we have a server address
                if idx == ClientState::LaunchPickRoom as usize {
                    if let Some(addr) = state.server_addr.take() {
                        match EnetClient::new(addr) {
                            Ok(client) => {
                                println!("Client: Successfully connected!");
                                net = Some(client);
                                ps1.octr_mut().driver_id = 0xFF;
                                ps1.octr_mut().current_state = ClientState::LaunchPickRoom as i32;
                                continue;
                            }
                            Err(e) => {
                                println!("Error: Failed to connect! ({})", e);
                                ps1.octr_mut().current_state = ClientState::LaunchPickServer as i32;
                                continue;
                            }
                        }
                    }
                }

                match net.as_mut() {
                    Some(ref mut n) => {
                        STATE_FUNCTIONS[idx](&ps1, n, &mut state);
                    }
                    None => {
                        STATE_FUNCTIONS[idx](&ps1, &mut EnetClient::dummy(), &mut state);
                    }
                }
            }
        }

        // Process network messages
        if let Some(ref mut n) = net {
            process_new_messages(&ps1, n, &mut state);
        }

        // Frame sync
        frame_stall(&ps1);
    }
}

fn print_banner(show_name: bool) {
    print!("\x1b[0;32m");
    println!("   ____    _    ____  __  __  _____  _____    _    _   _ ");
    println!("  / ___|  / \\  / ___||  \\/  |/ _ \\ \\/ /_ _|  / \\  | \\ | |");
    println!(" | |  _  / _ \\ \\___ \\| |\\/| | | | \\  / | |  / _ \\ |  \\| |");
    println!(" | |_| |/ ___ \\ ___) | |  | | |_| /  \\ | | / ___ \\| |\\  |");
    println!("  \\____/_/   \\_\\____/|_|  |_|\\___/_/\\_\\___/_/   \\_\\_| \\_|");
    println!("                                                          ");
    print!("\x1b[0m");
    println!(" Gasmoxian Client (press CTRL + C to quit)");
    println!();
    if show_name {
        println!(" Welcome!");
    }
}

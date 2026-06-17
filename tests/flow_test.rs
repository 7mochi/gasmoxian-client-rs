use deku::DekuContainerRead;

use gasmoxian_client_rs_v2::{
    effect::Effect,
    protocol::{
        ClientState, GASMOXIAN_VERSION,
        server::{ClientStatus, Rooms},
    },
    ps1_memory::LOBBY_LEVEL_ID,
    ps1_snapshot::OnlineCtrSnapshot,
    state::{self, GameState, handlers},
};

fn make_snapshot() -> OnlineCtrSnapshot {
    OnlineCtrSnapshot {
        current_state: 0,
        page_number: 0,
        count_press_x: 0,
        driver_count: 0,
        driver_id: 0,
        locked_in_lap: 0,
        locked_in_level: 0,
        lap_id: 0,
        level_id: 0,
        is_booted_ps1: 0,
        locked_in_character: 0,
        locked_in_engine: 0,
        room_count: 0,
        drivers_ended_count: 0,
        server_country: 0,
        server_room: 0,
        server_lock_in1: 0,
        server_lock_in2: 0,
        planet_lev: 0,
        client_busy: 0,
        locked_in_special: 0,
        special: 0,
        warpclock: 0,
        finish_race_timer: 0,
        client_count: [0; 16],
        windows_client_sync: 0,
        locked_in_characters: [0; 8],
        locked_in_engines: [0; 8],
        engine_type: [0; 8],
        name_buffer: [[0; 12]; 8],
        psx_version: GASMOXIAN_VERSION as i32,
        pc_version: 0,
        server_version: 0,
        shoot: [gasmoxian_client_rs_v2::protocol::ShootSlot {
            juiced: 0,
            weapon: 0,
            flags: 0,
            now: 0,
        }; 8],
        frames_unsynced: 0,
        last_windows_client_sync: 0,
        ready_to_send: 0,
        auto_retry_join_room_index: 0,
        gamemodes: [false; 18],
        room_type: 0,
        room_type_locked: 0,
        room_password_sequence: [0; 8],
        password_entered: [0; 8],
        gamepad_hold: 0,
        psx_pointer: 0,
        slot_psx_pointers: [0; 8],
        loading_stage: 0xFFFFFFFF,
        game_mode: 0,
        cutscene_level_id: LOBBY_LEVEL_ID,
        character_id: 0,
        cheats: 0,
        now_secs: 1000.0,
        kart_position_x: 0,
        kart_position_y: 0,
        kart_position_z: 0,
        kart_angle: 0,
        kart_wumpa: 0,
        kart_reserves: 0,
        race_course_time: 0,
        race_best_lap: 0,
    }
}

#[test]
fn flow_enter_pid_to_pick_server() {
    // Simulate the PS1 booting up.
    let mut ctr = make_snapshot();
    ctr.is_booted_ps1 = 0;
    let _state = GameState::new();

    // Not booted yet.
    let effects = state::launch_enter_pid(&ctr);
    assert!(effects.is_empty());

    // PS1 boots.
    ctr.is_booted_ps1 = 1;
    let effects = state::launch_enter_pid(&ctr);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LaunchPickServer)))
    );
}

#[test]
fn flow_server_selection_to_room_list() {
    let mut ctr = make_snapshot();
    let mut state = GameState::new();
    let server_addrs = ["127.0.0.1:54321".parse().ok()];

    // Player selects a server.
    ctr.server_lock_in1 = 1;
    ctr.server_country = 0;

    let effects = state::launch_pick_server(&ctr, &mut state, &server_addrs);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetClientBusy(1)))
    );

    // Server sends Rooms message. (The connection and state transition
    // to LaunchPickRoom happens on the PS1 side; here we just verify
    // the handler processes the packet correctly.)
    let rooms_payload = [
        0x00, 0x10, 0x03, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let (_, rooms_msg) = Rooms::from_bytes((&rooms_payload, 0)).unwrap();
    let effects = handlers::rooms::handle(&ctr, &mut state, rooms_msg);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetRoomCount(0x10)))
    );
}

#[test]
fn flow_room_browser_to_join() {
    let mut ctr = make_snapshot();
    let mut state = GameState::new();

    // Poll for room updates: every 60 frames sends a refresh packet.
    state.race.count_frame = 59;
    let effects = state::launch_pick_room(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert_eq!(state.race.count_frame, 0);

    // Player locks in a room.
    ctr.server_lock_in2 = 1;
    ctr.server_room = 3;
    let effects = state::launch_pick_room(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert_eq!(state.connection.attempt, 1);

    // Server sends NewClient -> full state reset + name sent.
    ctr.driver_id = 1;
    state.lobby.username = "TestPlayer".into();
    let (_, client_msg) = ClientStatus::from_bytes((&[0x13, 0x04], 0)).unwrap();
    let effects = handlers::new_client::handle(&ctr, &mut state, client_msg);
    assert!(effects.iter().any(|e| matches!(e, Effect::SetDriverId(1))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyAssignRole)))
    );
}

#[test]
fn flow_character_to_engine_to_loading() {
    let mut ctr = make_snapshot();
    let mut state = GameState::new();

    // Player picks character 5, locked in.
    ctr.character_id = 5;
    ctr.locked_in_characters[0] = 1;
    let effects = state::lobby_character_pick(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyEnginePick)))
    );

    // Player picks engine 2, locked in.
    ctr.engine_type[0] = 2;
    ctr.locked_in_engines[0] = 1;
    let effects = state::lobby_engine_pick(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyWaitForLoading)))
    );
    assert!(!state.race.flags.lock_engine_and_character);
}

#[test]
fn flow_race_lifecycle() {
    let mut ctr = make_snapshot();
    let mut state = GameState::new();

    // Start loading.
    let effects = state::lobby_start_loading(&mut state);
    assert!(!state.race.flags.sent_start_race);
    assert!(!state.race.flags.sent_end_race);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetFinishRaceTimer(0)))
    );

    // Game wait for race: sends kart data and start race once.
    ctr.current_state = ClientState::GameWaitForRace as i32;
    let effects = state::game_wait_for_race(&ctr, &mut state);
    assert!(state.race.flags.sent_start_race);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SendUnsequenced(_)))
    );

    // Game start race: sends kart data.
    ctr.current_state = ClientState::GameStartRace as i32;
    let effects = state::game_start_race(&ctr, &mut state);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SendUnsequenced(_)))
    );

    // Game end race: sends end race once.
    ctr.race_course_time = 120000;
    ctr.race_best_lap = 30000;
    let effects = state::game_end_race(&ctr, &mut state);
    assert!(state.race.flags.sent_end_race);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::WriteRaceStats { slot: 0, stats: _ }))
    );
}

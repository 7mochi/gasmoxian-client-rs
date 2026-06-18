use gasmoxian_client_rs_v2::{
    effect::Effect,
    protocol::{ClientState, MAX_NAME_LENGTH, MAX_NUM_PLAYERS},
    ps1_memory::LOBBY_LEVEL_ID,
    ps1_snapshot::OnlineCtrSnapshot,
    state::{self, GameState},
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
        locked_in_characters: [0; MAX_NUM_PLAYERS],
        locked_in_engines: [0; MAX_NUM_PLAYERS],
        engine_type: [0; MAX_NUM_PLAYERS],
        name_buffer: [[0; MAX_NAME_LENGTH + 1]; MAX_NUM_PLAYERS],
        psx_version: 3,
        pc_version: 0,
        server_version: 0,
        shoot: [gasmoxian_client_rs_v2::protocol::ShootSlot {
            juiced: 0,
            weapon: 0,
            flags: 0,
            now: 0,
        }; MAX_NUM_PLAYERS],
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
        slot_psx_pointers: [0; MAX_NUM_PLAYERS],
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
fn launch_enter_pid_not_booted() {
    let ctr = make_snapshot();
    let effects = state::launch_enter_pid(&ctr);
    assert!(effects.is_empty());
}

#[test]
fn launch_enter_pid_booted() {
    let mut ctr = make_snapshot();
    ctr.is_booted_ps1 = 1;
    let effects = state::launch_enter_pid(&ctr);
    assert_eq!(effects.len(), 4);
    assert!(matches!(
        effects[3],
        Effect::SetState(ClientState::LaunchPickServer)
    ));
}

#[test]
fn launch_pick_server_cutscene_wrong() {
    let mut ctr = make_snapshot();
    ctr.cutscene_level_id = 0;
    let effects = state::launch_pick_server(&ctr, &mut GameState::new(), &[None]);
    assert!(effects.is_empty());
}

#[test]
fn launch_pick_server_loading() {
    let mut ctr = make_snapshot();
    ctr.loading_stage = 0;
    let effects = state::launch_pick_server(&ctr, &mut GameState::new(), &[None]);
    assert!(effects.is_empty());
}

#[test]
fn launch_pick_server_not_locked() {
    let ctr = make_snapshot();
    let effects = state::launch_pick_server(&ctr, &mut GameState::new(), &[None]);
    assert!(effects.is_empty());
}

#[test]
fn launch_pick_server_ready() {
    let mut ctr = make_snapshot();
    ctr.server_lock_in1 = 1;
    ctr.server_country = 0;
    let mut state = GameState::new();
    let addr = "127.0.0.1:54321".parse().ok();
    let effects = state::launch_pick_server(&ctr, &mut state, &[addr]);
    assert_eq!(state.connection.static_server_id, 0);
    assert!(state.connection.server_addr.is_some());
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetClientBusy(1)))
    );
}

#[test]
fn launch_pick_room_poll() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    state.race.count_frame = 59;
    let effects = state::launch_pick_room(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert_eq!(state.race.count_frame, 0);
}

#[test]
fn launch_pick_room_join() {
    let mut ctr = make_snapshot();
    ctr.server_lock_in2 = 1;
    ctr.server_room = 3;
    let mut state = GameState::new();
    let effects = state::launch_pick_room(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert!(state.connection.attempt == 1);
}

#[test]
fn lobby_assign_role_guest() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    state.connection.driver_id = 2;
    let effects = state::lobby_assign_role(&ctr, &mut state);
    assert!(effects.is_empty());
}

#[test]
fn lobby_assign_role_host() {
    let mut ctr = make_snapshot();
    ctr.room_type_locked = 1;
    ctr.room_type = 0;
    let effects = state::lobby_assign_role(&ctr, &mut GameState::new());
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
}

#[test]
fn lobby_character_pick_change() {
    let mut ctr = make_snapshot();
    ctr.character_id = 5;
    let mut state = GameState::new();
    let effects = state::lobby_character_pick(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
}

#[test]
fn lobby_character_pick_lock() {
    let mut ctr = make_snapshot();
    ctr.character_id = 3;
    ctr.locked_in_characters[0] = 1;
    let mut state = GameState::new();
    let effects = state::lobby_character_pick(&ctr, &mut state);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyEnginePick)))
    );
}

#[test]
fn lobby_engine_pick_lock() {
    let mut ctr = make_snapshot();
    ctr.engine_type[0] = 2;
    ctr.locked_in_engines[0] = 1;
    let mut state = GameState::new();
    let effects = state::lobby_engine_pick(&ctr, &mut state);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyWaitForLoading)))
    );
    assert!(!state.race.flags.lock_engine_and_character);
}

#[test]
fn lobby_host_track_pick_send() {
    let mut ctr = make_snapshot();
    ctr.locked_in_lap = 1;
    ctr.level_id = 7;
    ctr.lap_id = 2;
    let effects = state::lobby_host_track_pick(&ctr, &mut GameState::new());
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbySpecialPick)))
    );
}

#[test]
fn lobby_special_pick_send() {
    let mut ctr = make_snapshot();
    ctr.locked_in_special = 1;
    ctr.gamemodes[1] = true; // Mirror
    let effects = state::lobby_special_pick(&ctr, &mut GameState::new());
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyCharacterPick)))
    );
}

#[test]
fn disconnect_dselect() {
    let mut ctr = make_snapshot();
    ctr.gamepad_hold = 0x2000;
    let mut state = GameState::new();
    let effects = state::disconnect(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::DisconnectNow)));
    assert!(effects.iter().any(|e| matches!(e, Effect::SetStateRaw(-1))));
}

#[test]
fn disconnect_not_pressed() {
    let ctr = make_snapshot();
    let effects = state::disconnect(&ctr, &mut GameState::new());
    assert!(effects.is_empty());
}

#[test]
fn afk_timer_starts_when_locked() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    state.race.flags.lock_engine_and_character = true;
    let effects = state::afk_timer(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::LogDebug(_))));
    assert!(state.race.time_start > 0.0);
}

#[test]
fn afk_timer_timeout() {
    let mut ctr = make_snapshot();
    ctr.now_secs = 2000.0;
    let mut state = GameState::new();
    state.race.flags.lock_engine_and_character = true;
    state.race.time_start = 1000.0;
    let effects = state::afk_timer(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::DisconnectNow)));
    assert!(effects.iter().any(|e| matches!(e, Effect::SetStateRaw(-1))));
}

#[test]
fn game_wait_for_race_sends_kart() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let effects = state::game_wait_for_race(&ctr, &mut state);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SendUnsequenced(_)))
    );
}

#[test]
fn game_wait_for_race_sends_start_race_once() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let effects = state::game_wait_for_race(&ctr, &mut state);
    // Camera fly-in not done (game_mode & 0x40) != 0 means fly-in in progress,
    // but our default game_mode = 0 means fly-in is done, so start race should be sent.
    assert!(state.race.flags.sent_start_race);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
}

#[test]
fn game_wait_for_race_start_race_once_only() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let _ = state::game_wait_for_race(&ctr, &mut state);
    let effects2 = state::game_wait_for_race(&ctr, &mut state);
    // Second call should not resend start race
    assert!(state.race.flags.sent_start_race);
    let reliable_count = effects2
        .iter()
        .filter(|e| matches!(e, Effect::SendReliable(_)))
        .count();
    // Only the kart SendUnsequenced, no StartRace
    assert_eq!(reliable_count, 0);
}

#[test]
fn game_end_race_sends_end_race_once() {
    let mut ctr = make_snapshot();
    ctr.race_course_time = 120000;
    ctr.race_best_lap = 30000;
    let mut state = GameState::new();
    let effects = state::game_end_race(&ctr, &mut state);
    assert!(state.race.flags.sent_end_race);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
}

#[test]
fn game_end_race_second_call_noop() {
    let mut ctr = make_snapshot();
    ctr.race_course_time = 120000;
    let mut state = GameState::new();
    let _ = state::game_end_race(&ctr, &mut state);
    // snapshot is now borrowed immutably for the second call
    let effects2 = state::game_end_race(&ctr, &mut state);
    let reliable_count = effects2
        .iter()
        .filter(|e| matches!(e, Effect::SendReliable(_)))
        .count();
    assert_eq!(reliable_count, 0);
}

#[test]
fn game_start_race_sends_kart() {
    let ctr = make_snapshot();
    let effects = state::game_start_race(&ctr, &mut GameState::new());
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SendUnsequenced(_)))
    );
}

#[test]
fn game_end_race_sends_end_race_sets_stats() {
    let mut ctr = make_snapshot();
    ctr.race_course_time = 120000;
    ctr.race_best_lap = 30000;
    let mut state = GameState::new();
    let effects = state::game_end_race(&ctr, &mut state);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::WriteRaceStats { slot: 0, stats: _ }))
    );
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetDriversEndedCount(1)))
    );
}

#[test]
fn launch_enter_password_sends_ping() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    state.race.count_frame = 59;
    let effects = state::launch_enter_password(&ctr, &mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::Ping)));
    assert_eq!(state.race.count_frame, 0);
}

#[test]
fn launch_enter_password_sends_password() {
    let mut ctr = make_snapshot();
    ctr.room_password_sequence = [1, 2, 3, 4, 5, 6, 7, 8];
    ctr.password_entered[7] = 1;
    let mut state = GameState::new();
    let effects = state::launch_enter_password(&ctr, &mut state);
    assert!(state.race.flags.password_sent);
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
}

#[test]
fn launch_error_returns_empty() {
    let effects = state::launch_error();
    assert!(effects.is_empty());
}

#[test]
fn lobby_guest_track_wait_resets_selection() {
    let mut state = GameState::new();
    state.previous_selection.character_id = Some(5);
    state.previous_selection.engine_type = Some(1);
    let effects = state::lobby_guest_track_wait(&mut state);
    assert!(state.previous_selection.character_id.is_none());
    assert!(state.previous_selection.engine_type.is_none());
    assert!(effects.is_empty());
}

#[test]
fn lobby_wait_for_loading_returns_empty() {
    let effects = state::lobby_wait_for_loading();
    assert!(effects.is_empty());
}

#[test]
fn lobby_start_loading_resets_flags() {
    let mut state = GameState::new();
    state.race.flags.sent_start_race = true;
    state.race.flags.sent_end_race = true;
    let effects = state::lobby_start_loading(&mut state);
    assert!(!state.race.flags.sent_start_race);
    assert!(!state.race.flags.sent_end_race);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetFinishRaceTimer(0)))
    );
}

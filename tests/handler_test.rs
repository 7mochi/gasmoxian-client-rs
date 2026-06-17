use deku::DekuContainerRead;

use gasmoxian_client_rs_v2::{
    effect::Effect,
    protocol::{
        ClientState, GASMOXIAN_VERSION,
        server::{
            Character, ClientStatus, EndRace, Engine, FinishTimer, Kart, Name, RoomType, Rooms,
            Special, Track, WarpClock, Weapon,
        },
    },
    ps1_memory::{GAMEPAD_BASE, LOBBY_LEVEL_ID},
    ps1_snapshot::OnlineCtrSnapshot,
    state::{GameState, handlers},
};

fn make_snapshot() -> OnlineCtrSnapshot {
    OnlineCtrSnapshot {
        current_state: ClientState::LaunchPickRoom as i32,
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
fn handle_rooms_version_ok() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let payload = [
        0x00, 0x10, 0x03, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let (_, msg) = Rooms::from_bytes((&payload, 0)).unwrap();
    let effects = handlers::rooms::handle(&ctr, &mut state, msg);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetRoomCount(0x10)))
    );
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetServerVersion(3)))
    );
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LaunchError)))
    );
}

#[test]
fn handle_rooms_version_mismatch() {
    let mut ctr = make_snapshot();
    ctr.psx_version = 99;
    let mut state = GameState::new();
    let payload = [
        0x00, 0x10, 0x03, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let (_, msg) = Rooms::from_bytes((&payload, 0)).unwrap();
    let effects = handlers::rooms::handle(&ctr, &mut state, msg);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LaunchError)))
    );
}

#[test]
fn handle_room_type_normal() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let (_, msg) = RoomType::from_bytes((&[0x01, 0x00, 0x00], 0)).unwrap();
    let effects = handlers::room_type::handle(&ctr, &mut state, msg);
    assert!(effects.iter().any(|e| matches!(e, Effect::SetRoomType(0))));
}

#[test]
fn handle_room_type_password() {
    let mut ctr = make_snapshot();
    ctr.current_state = ClientState::LaunchPickRoom as i32;
    let mut state = GameState::new();
    let (_, msg) = RoomType::from_bytes((&[0x01, 0x02, 0x00], 0)).unwrap();
    let effects = handlers::room_type::handle(&ctr, &mut state, msg);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LaunchEnterPassword)))
    );
}

#[test]
fn handle_new_client_resets_and_sends_name() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    state.lobby.username = "TestPlayer".into();
    let (_, msg) = ClientStatus::from_bytes((&[0x13, 0x04], 0)).unwrap();
    let effects = handlers::new_client::handle(&ctr, &mut state, msg);
    assert!(effects.iter().any(|e| matches!(e, Effect::SetDriverId(1))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetDriverCount(4)))
    );
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyAssignRole)))
    );
    // Name message should be sent
    assert!(effects.iter().any(|e| matches!(e, Effect::SendReliable(_))));
}

#[test]
fn handle_name_other_player() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 1; // not the sender
    let mut state = GameState::new();
    let payload = [
        0x04, 0x40, 0x58, 0x6e, 0x69, 0x74, 0x72, 0x6f, 0x36, 0x37, 0x00, 0x00, 0x00, 0x00,
    ];
    let (_, msg) = Name::from_bytes((&payload, 0)).unwrap();
    let effects = handlers::name::handle(&ctr, &mut state, msg);
    // client_id=0, driver_id=1 => slot = 0+1 = 1 (client_id < driver_id)
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetDriverCount(4)))
    );
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetNameBuffer { slot: 1, data: _ }))
    );
}

#[test]
fn handle_name_self_is_noop() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 0;
    let mut state = GameState::new();
    let payload = [
        0x04, 0x00, 0x58, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let (_, msg) = Name::from_bytes((&payload, 0)).unwrap();
    let effects = handlers::name::handle(&ctr, &mut state, msg);
    assert!(effects.is_empty());
}

#[test]
fn handle_track_sets_level() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let (_, msg) = Track::from_bytes((&[0x05, 0x0a, 0x00], 0)).unwrap();
    let effects = handlers::track::handle(&ctr, &mut state, msg);
    assert!(effects.iter().any(|e| matches!(e, Effect::SetLevelId(10))));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbySpecialPick)))
    );
}

#[test]
fn handle_special_sets_gamemodes() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let payload = [
        0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x00, 0x00,
    ];
    let (_, msg) = Special::from_bytes((&payload, 0)).unwrap();
    let effects = handlers::special::handle(&ctr, &mut state, msg);
    assert!(effects.iter().any(|e| matches!(
        e,
        Effect::SetGamemode {
            index: 0,
            value: true
        }
    )));
    assert!(effects.iter().any(|e| matches!(
        e,
        Effect::SetGamemode {
            index: 5,
            value: true
        }
    )));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyCharacterPick)))
    );
}

#[test]
fn handle_character_other_player() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 0;
    let mut state = GameState::new();
    let (_, msg) = Character::from_bytes((&[0x17, 0x08], 0)).unwrap();
    let expected_addr = 0x80086e84 + 2;
    let effects = handlers::character::handle(&ctr, &mut state, msg);
    assert!(effects.iter().any(|e| {
        if let Effect::WriteU16(addr, val) = e {
            *addr == expected_addr && *val == 8
        } else {
            false
        }
    }));
}

#[test]
fn handle_character_self_is_noop() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 1;
    let mut state = GameState::new();
    let (_, msg) = Character::from_bytes((&[0x17, 0x08], 0)).unwrap();
    let effects = handlers::character::handle(&ctr, &mut state, msg);
    assert!(effects.is_empty());
}

#[test]
fn handle_engine_other_player() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 0;
    let mut state = GameState::new();
    let (_, msg) = Engine::from_bytes((&[0x18, 0x00, 0x00], 0)).unwrap();
    let effects = handlers::engine::handle(&ctr, &mut state, msg);
    // client_id=1, driver_id=0 => slot=1 (client_id < driver_id is false)
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetEngineType { slot: 1, value: 0 }))
    );
}

#[test]
fn handle_start_loading() {
    let effects = handlers::start_loading::handle();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LobbyStartLoading)))
    );
}

#[test]
fn handle_start_race() {
    let effects = handlers::start_race::handle();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::GameStartRace)))
    );
}

#[test]
fn handle_race_data_other_player() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 0;
    ctr.current_state = ClientState::GameWaitForRace as i32;
    let payload = [0x0b, 0x72, 0x60, 0x00, 0x7c, 0x0c, 0x00, 0x00, 0x9e, 0x17];
    let mut state = GameState::new();
    let (_, msg) = Kart::from_bytes((&payload, 0)).unwrap();

    let expected_gp = GAMEPAD_BASE + 2 * 0x50;
    let effects = handlers::race_data::handle(&ctr, &mut state, msg);
    assert!(effects.iter().any(|e| {
        if let Effect::WriteU32(addr, _) = e {
            *addr == expected_gp
        } else {
            false
        }
    }));
}

#[test]
fn handle_race_data_before_race_is_noop() {
    let ctr = make_snapshot();
    let mut state = GameState::new();
    let payload = [0x0b, 0x72, 0x60, 0x00, 0x7c, 0x0c, 0x00, 0x00, 0x9e, 0x17];
    let (_, msg) = Kart::from_bytes((&payload, 0)).unwrap();
    let effects = handlers::race_data::handle(&ctr, &mut state, msg);
    assert!(effects.is_empty());
}

#[test]
fn handle_weapon_other_player() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 0;
    let mut state = GameState::new();
    let (_, msg) = Weapon::from_bytes((&[0x1c, 0x05], 0)).unwrap();
    let effects = handlers::weapon::handle(&ctr, &mut state, msg);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetShoot { slot: 1, shoot: _ }))
    );
}

#[test]
fn handle_warp_clock_change() {
    let mut ctr = make_snapshot();
    ctr.warpclock = 0;
    let mut state = GameState::new();
    let (_, msg) = WarpClock::from_bytes((&[0x1d], 0)).unwrap();
    let effects = handlers::warp_clock::handle(&ctr, &mut state, msg);
    assert!(effects.iter().any(|e| matches!(e, Effect::SetWarpclock(1))));
}

#[test]
fn handle_finish_timer_change() {
    let mut state = GameState::new();
    state.previous.finish_timer = Some(0);
    let ctr = make_snapshot();
    let (_, msg) = FinishTimer::from_bytes((&[0x0e, 0x1e], 0)).unwrap();
    let effects = handlers::finish_timer::handle(&ctr, &mut state, msg);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetFinishRaceTimer(30)))
    );
}

#[test]
fn handle_end_race_other_player() {
    let mut ctr = make_snapshot();
    ctr.driver_id = 0;
    ctr.drivers_ended_count = 0;
    let mut state = GameState::new();
    let payload = [
        0x0f, 0x04, 0x00, 0x00, 0x40, 0xe2, 0x01, 0x00, 0x35, 0x34, 0x01, 0x00,
    ];
    let (_, msg) = EndRace::from_bytes((&payload, 0)).unwrap();
    let effects = handlers::end_race::handle(&ctr, &mut state, msg);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetDriversEndedCount(1)))
    );
}

#[test]
fn handle_password_rejected() {
    let mut state = GameState::new();
    let effects = handlers::password_rejected::handle(&mut state);
    assert!(effects.iter().any(|e| matches!(e, Effect::DisconnectNow)));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::SetState(ClientState::LaunchPickRoom)))
    );
    assert!(!state.race.flags.password_sent);
}

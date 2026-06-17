use deku::DekuContainerRead;

use gasmoxian_client_rs_v2::protocol::server::{
    Character, ClientStatus, EndRace, Engine, FinishTimer, Kart, Name, RoomType, Rooms, Special,
    Track, WarpClock, Weapon,
};

const ROOMS: &[u8] = &[
    0x00, 0x10, 0x03, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const ROOM_TYPE: &[u8] = &[0x01, 0x00, 0x00];
const NEW_CLIENT: &[u8] = &[0x13, 0x04];
const NAME: &[u8] = &[
    0x04, 0x40, 0x58, 0x6e, 0x69, 0x74, 0x72, 0x6f, 0x36, 0x37, 0x00, 0x00, 0x00, 0x00,
];
const TRACK: &[u8] = &[0x05, 0x0a, 0x00];
const SPECIAL: &[u8] = &[
    0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00,
    0x01, 0x00, 0x00,
];
const CHARACTER_OTHER: &[u8] = &[0x17, 0x08];
const CHARACTER_SELF: &[u8] = &[0x07, 0x00];
const ENGINE: &[u8] = &[0x18, 0x00, 0x00];
const KART: &[u8] = &[0x0b, 0x72, 0x60, 0x00, 0x7c, 0x0c, 0x00, 0x00, 0x9e, 0x17];
const WEAPON: &[u8] = &[0x1c, 0x04];
const WARPCLOCK: &[u8] = &[0x1d];
const FINISH_TIMER: &[u8] = &[0x0e, 0x1e];
const END_RACE: &[u8] = &[
    0x0f, 0x04, 0x00, 0x00, 0x40, 0xe2, 0x01, 0x00, 0x35, 0x34, 0x01, 0x00,
];

#[test]
fn deserialize_rooms() {
    let (_, msg) = Rooms::from_bytes((ROOMS, 0)).expect("Rooms deserialize failed");
    assert_eq!(msg.room_count, 0x10);
    assert_eq!(msg.version, 3);
}

#[test]
fn deserialize_room_type() {
    let (_, msg) = RoomType::from_bytes((ROOM_TYPE, 0)).expect("RoomType deserialize failed");
    assert_eq!(msg.room_type, 0);
    assert_eq!(msg.r_type_locked, 0);
}

#[test]
fn deserialize_new_client() {
    let (_, msg) =
        ClientStatus::from_bytes((NEW_CLIENT, 0)).expect("ClientStatus deserialize failed");
    assert_eq!(msg.client_id, 1);
    assert_eq!(msg.client_count, 4);
}

#[test]
fn deserialize_name() {
    let (_, msg) = Name::from_bytes((NAME, 0)).expect("Name deserialize failed");
    assert_eq!(msg.client_id, 0);
    assert_eq!(msg.client_count, 4);
    assert_eq!(
        msg.username[..7],
        [0x58, 0x6e, 0x69, 0x74, 0x72, 0x6f, 0x36]
    );
    assert_eq!(msg.username[7], 0x37);
}

#[test]
fn deserialize_track() {
    let (_, msg) = Track::from_bytes((TRACK, 0)).expect("Track deserialize failed");
    assert_eq!(msg.track_id, 10);
    assert_eq!(msg.lap_id, 0);
}

#[test]
fn deserialize_special() {
    let (_, msg) = Special::from_bytes((SPECIAL, 0)).expect("Special deserialize failed");
    assert!(msg.gamemodes[0]); // Normal
    assert!(!msg.gamemodes[1]); // Mirror
    assert!(msg.gamemodes[5]); // RetroFueled
    assert!(msg.gamemodes[10]); // Shortcutless
    assert!(msg.gamemodes[13]); // ItemChaos
    assert!(msg.gamemodes[15]); // SurvivalTimer
    assert!(!msg.gamemodes[17]); // WallDrive
}

#[test]
fn deserialize_character_other() {
    let (_, msg) =
        Character::from_bytes((CHARACTER_OTHER, 0)).expect("Character deserialize failed");
    assert_eq!(msg.client_id, 1);
    assert!(!msg.locked_in);
    assert_eq!(msg.character_id, 8);
}

#[test]
fn deserialize_character_self() {
    let (_, msg) =
        Character::from_bytes((CHARACTER_SELF, 0)).expect("Character deserialize failed");
    assert_eq!(msg.client_id, 0);
    assert!(!msg.locked_in);
    assert_eq!(msg.character_id, 0);
}

#[test]
fn deserialize_engine() {
    let (_, msg) = Engine::from_bytes((ENGINE, 0)).expect("Engine deserialize failed");
    assert_eq!(msg.client_id, 1);
    assert_eq!(msg.engine_type, 0);
    assert!(!msg.locked_in);
}

#[test]
fn deserialize_kart() {
    let (_, msg) = Kart::from_bytes((KART, 0)).expect("Kart deserialize failed");
    assert_eq!(msg.wumpa, 0);
    assert!(!msg.reserves);
    assert_eq!(msg.client_id, 2);
    assert_eq!(msg.kart_rotation1, 14);
    assert_eq!(msg.kart_rotation2, 0x60);
    assert_eq!(msg.button_hold, 0);
    assert_eq!(msg.position_x, 3196);
    assert_eq!(msg.position_y, 0);
    assert_eq!(msg.position_z, 6046);
}

#[test]
fn deserialize_weapon() {
    let (_, msg) = Weapon::from_bytes((WEAPON, 0)).expect("Weapon deserialize failed");
    assert!(!msg.juiced);
    assert_eq!(msg.flags, 0);
    assert_eq!(msg.weapon, 4);
}

#[test]
fn deserialize_warp_clock() {
    let (_, msg) = WarpClock::from_bytes((WARPCLOCK, 0)).expect("WarpClock deserialize failed");
    assert_eq!(msg.warp_clock, 1);
}

#[test]
fn deserialize_finish_timer() {
    let (_, msg) =
        FinishTimer::from_bytes((FINISH_TIMER, 0)).expect("FinishTimer deserialize failed");
    assert_eq!(msg.finish_timer, 30);
}

#[test]
fn deserialize_end_race() {
    let (_, msg) = EndRace::from_bytes((END_RACE, 0)).expect("EndRace deserialize failed");
    assert_eq!(msg.client_id, 4);
    assert_eq!(msg.course_time, 123456);
    assert_eq!(msg.lap_time, 78901);
}

pub struct EverythingKart {
    pub client_id: u8,
    pub wumpa: u8,
    pub bool_reserves: bool,
    pub kart_rot1: u8,
    pub kart_rot2: u8,
    pub button_hold: u8,
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
}
impl EverythingKart {
    pub fn from_bytes(data: &[u8]) -> Self {
        EverythingKart {
            client_id: data[1] & 0x07,
            wumpa: (data[0] >> 4) & 0x07,
            bool_reserves: (data[0] >> 7) != 0,
            kart_rot1: (data[1] >> 3) & 0x1F,
            kart_rot2: data[2],
            button_hold: data[3],
            pos_x: i16::from_le_bytes([data[4], data[5]]),
            pos_y: i16::from_le_bytes([data[6], data[7]]),
            pos_z: i16::from_le_bytes([data[8], data[9]]),
        }
    }
}

pub struct Header {
    pub msg_type: u8,
}
impl Header {
    pub fn from_bytes(data: &[u8]) -> Self {
        Header {
            msg_type: data[0] & 0x0F,
        }
    }
}

pub struct MessageClientStatus {
    pub client_id: u8,
    pub num_clients: u8,
}
impl MessageClientStatus {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageClientStatus {
            client_id: (data[0] >> 4) & 0x0F,
            num_clients: data[1] & 0x0F,
        }
    }
}

pub struct MessageRoomType {
    pub room_type: u8,
    pub r_type_locked: u8,
}
impl MessageRoomType {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageRoomType {
            room_type: data[1],
            r_type_locked: data[2],
        }
    }
}

pub struct MessageRooms {
    pub num_rooms: u8,
    pub version: u16,
    pub client_count: [i8; 16],
}
impl MessageRooms {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut client_count = [0i8; 16];
        for i in 0..8 {
            client_count[i * 2] = (data[4 + i] & 0x0F) as i8;
            client_count[i * 2 + 1] = (data[4 + i] >> 4) as i8;
        }
        MessageRooms {
            num_rooms: data[1],
            version: u16::from_le_bytes([data[2], data[3]]),
            client_count,
        }
    }
}

pub struct MessagePasswordRejected;
impl MessagePasswordRejected {
    pub fn from_bytes(_data: &[u8]) -> Self {
        MessagePasswordRejected
    }
}

pub struct MessageName {
    pub client_id: u8,
    pub num_clients: u8,
    pub name: [u8; 12],
}
impl MessageName {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut name = [0u8; 12];
        name.copy_from_slice(&data[2..14]);
        MessageName {
            client_id: data[1] & 0x0F,
            num_clients: data[1] >> 4,
            name,
        }
    }
}

pub struct MessageTrack {
    pub track_id: u8,
    pub lap_id: u8,
}
impl MessageTrack {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageTrack {
            track_id: data[1] & 0x1F,
            lap_id: data[2],
        }
    }
}

pub struct MessageSpecial {
    pub gamemodes: [bool; 18],
}
impl MessageSpecial {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut gamemodes = [false; 18];
        for i in 0..18 {
            gamemodes[i] = data[1 + i] != 0;
        }
        MessageSpecial { gamemodes }
    }
}

pub struct MessageCharacter {
    pub client_id: u8,
    pub bool_locked_in: bool,
    pub character_id: u8,
}
impl MessageCharacter {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageCharacter {
            client_id: (data[0] >> 4) & 0x07,
            bool_locked_in: (data[0] >> 7) & 1 != 0,
            character_id: data[1] & 0x0F,
        }
    }
}

pub struct MessageEngine {
    pub client_id: u8,
    pub enginetype: u8,
    pub bool_locked_in: bool,
}
impl MessageEngine {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageEngine {
            client_id: (data[0] >> 4) & 0x07,
            enginetype: data[1] & 0x0F,
            bool_locked_in: (data[1] >> 4) & 1 != 0,
        }
    }
}

pub struct MessageWeapon {
    pub client_id: u8,
    pub juiced: bool,
    pub weapon: u8,
    pub flags: u8,
}
impl MessageWeapon {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageWeapon {
            client_id: (data[0] >> 4) & 0x07,
            juiced: (data[0] >> 7) & 1 != 0,
            weapon: data[1] & 0x0F,
            flags: (data[1] >> 4) & 0x03,
        }
    }
}

pub struct MessageEndRace {
    pub client_id: u8,
    pub course_time: i32,
    pub lap_time: i32,
}
impl MessageEndRace {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageEndRace {
            client_id: data[1] & 0x0F,
            course_time: i32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            lap_time: i32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        }
    }
}

pub struct MessageWarpclock {
    pub warpclock: u8,
}
impl MessageWarpclock {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageWarpclock {
            warpclock: (data[0] >> 6) & 0x03,
        }
    }
}

pub struct MessageFinishTimer {
    pub finish_timer: u8,
}
impl MessageFinishTimer {
    pub fn from_bytes(data: &[u8]) -> Self {
        MessageFinishTimer {
            finish_timer: data[1] & 0x3F,
        }
    }
}

use super::ClientMsg;

pub struct EverythingKart {
    pub wumpa: u8,
    pub reserves: bool,
    pub kart_rot1: u8,
    pub kart_rot2: u8,
    pub button_hold: u8,
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
}
impl EverythingKart {
    pub fn to_bytes(&self) -> [u8; 10] {
        let b0 = (ClientMsg::RaceData as u8) & 0x0F
            | (self.wumpa & 0x07) << 4
            | (self.reserves as u8) << 7;
        [
            b0,
            self.kart_rot1,
            self.kart_rot2,
            self.button_hold,
            self.pos_x.to_le_bytes()[0],
            self.pos_x.to_le_bytes()[1],
            self.pos_y.to_le_bytes()[0],
            self.pos_y.to_le_bytes()[1],
            self.pos_z.to_le_bytes()[0],
            self.pos_z.to_le_bytes()[1],
        ]
    }
}

pub struct MessageWeapon {
    pub juiced: bool,
    pub flags: u8,
    pub weapon: u8,
}
impl MessageWeapon {
    pub fn to_bytes(&self) -> [u8; 2] {
        let b0 = (ClientMsg::Weapon as u8) & 0x0F
            | (self.juiced as u8) << 4
            | 0 << 5
            | (self.flags & 0x03) << 6;
        let b1 = self.weapon & 0x0F;
        [b0, b1]
    }
}

pub struct Header;
impl Header {
    pub fn to_bytes() -> [u8; 1] {
        [(ClientMsg::StartRace as u8) & 0x0F]
    }
}

pub struct MessageRoom {
    pub room: u8,
}
impl MessageRoom {
    pub fn to_bytes(&self) -> [u8; 2] {
        [(ClientMsg::JoinRoom as u8) & 0x0F, self.room]
    }
}

pub struct MessageRoomType {
    pub room_type: u8,
    pub r_type_locked: u8,
}
impl MessageRoomType {
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (ClientMsg::RoomType as u8) & 0x0F,
            self.room_type,
            self.r_type_locked,
        ]
    }
}

pub struct MessageRoomTypePassword {
    pub room_type: u8,
    pub r_type_locked: u8,
    pub seq: [u8; 8],
}
impl MessageRoomTypePassword {
    pub fn to_bytes(&self) -> [u8; 11] {
        let mut buf = [0u8; 11];
        buf[0] = (ClientMsg::RoomType as u8) & 0x0F;
        buf[1] = self.room_type;
        buf[2] = self.r_type_locked;
        buf[3..11].copy_from_slice(&self.seq);
        buf
    }
}

pub struct MessagePassword {
    pub seq: [u8; 8],
}
impl MessagePassword {
    pub fn to_bytes(&self) -> [u8; 9] {
        let mut buf = [0u8; 9];
        buf[0] = (ClientMsg::Password as u8) & 0x0F;
        buf[1..9].copy_from_slice(&self.seq);
        buf
    }
}

pub struct MessageName {
    pub name: [u8; 12],
}
impl MessageName {
    pub fn to_bytes(&self) -> [u8; 13] {
        let mut buf = [0u8; 13];
        buf[0] = (ClientMsg::Name as u8) & 0x0F;
        buf[1..13].copy_from_slice(&self.name);
        buf
    }
}

pub struct MessageTrack {
    pub track_id: u8,
    pub lap_id: u8,
}
impl MessageTrack {
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (ClientMsg::Track as u8) & 0x0F,
            (self.track_id & 0x1F) | ((self.lap_id & 0x07) << 5),
            (self.lap_id >> 3) & 0x1F,
        ]
    }
}

pub struct MessageSpecial {
    pub gamemodes: [bool; 18],
}
impl MessageSpecial {
    pub fn to_bytes(&self) -> [u8; 19] {
        let mut buf = [0u8; 19];
        buf[0] = (ClientMsg::Special as u8) & 0x0F;
        for i in 0..18 {
            buf[1 + i] = self.gamemodes[i] as u8;
        }
        buf
    }
}

pub struct MessageCharacter {
    pub character_id: u8,
    pub bool_locked_in: bool,
}
impl MessageCharacter {
    pub fn to_bytes(&self) -> [u8; 2] {
        [
            (ClientMsg::Character as u8) & 0x0F | (self.character_id & 0x0F) << 4,
            self.bool_locked_in as u8,
        ]
    }
}

pub struct MessageEngine {
    pub enginetype: u8,
    pub bool_locked_in: bool,
}
impl MessageEngine {
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (ClientMsg::Engine as u8) & 0x0F,
            (self.enginetype & 0x0F) | ((self.bool_locked_in as u8) & 1) << 4,
            0,
        ]
    }
}

pub struct MessageWarpclock {
    pub warpclock: u8,
}
impl MessageWarpclock {
    pub fn to_bytes(&self) -> [u8; 1] {
        [((ClientMsg::Warpclock as u8) & 0x0F) | ((self.warpclock & 0x03) << 6)]
    }
}

pub struct MessageFinishTimer {
    pub finish_timer: u8,
}
impl MessageFinishTimer {
    pub fn to_bytes(&self) -> [u8; 2] {
        [
            (ClientMsg::FinishTimer as u8) & 0x0F,
            self.finish_timer & 0x3F,
        ]
    }
}

pub struct MessageEndRace {
    pub course_time: i32,
    pub lap_time: i32,
}
impl MessageEndRace {
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0] = (ClientMsg::EndRace as u8) & 0x0F;
        buf[4..8].copy_from_slice(&self.course_time.to_le_bytes());
        buf[8..12].copy_from_slice(&self.lap_time.to_le_bytes());
        buf
    }
}

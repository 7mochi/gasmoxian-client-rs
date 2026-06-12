// mods/Windows/Gasmoxian/Network_PC/GClient/GASMOX_CLIENT.cpp:872-1031

pub struct ServerInfo {
    pub address: &'static str,
    pub port: u16,
}

pub const SERVERS: [ServerInfo; 5] = [
    ServerInfo {
        address: "mednafen-peru2.ddns.net",
        port: 54321,
    },
    ServerInfo {
        address: "mednafen-us.ddns.net",
        port: 54321,
    },
    ServerInfo {
        address: "ctr.ryu7w7.xyz",
        port: 5727,
    },
    ServerInfo {
        address: "gasmoxbr.duckdns.org",
        port: 5029,
    },
    ServerInfo {
        address: "38.47.191.253",
        port: 7777,
    },
];

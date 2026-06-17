//! Hardcoded server endpoints and DNS resolution.
//!
//! [`SERVERS`] contains the five known Gasmoxian community servers.
//! Each entry has a display name and an endpoint string resolved
//! at runtime via [`ServerInfo::resolve`].

use std::net::{SocketAddr, ToSocketAddrs};

use crate::console;

pub struct ServerInfo {
    pub name: &'static str,
    pub endpoint: &'static str,
}

pub const SERVERS: &[ServerInfo] = &[
    ServerInfo {
        name: "Peru",
        endpoint: "mednafen-peru2.ddns.net:54321",
    },
    ServerInfo {
        name: "US East",
        endpoint: "mednafen-us.ddns.net:54321",
    },
    ServerInfo {
        name: "Chile",
        endpoint: "ctr.ryu7w7.xyz:5727",
    },
    ServerInfo {
        name: "Brasil",
        endpoint: "gasmoxbr.duckdns.org:5029",
    },
    ServerInfo {
        name: "Asia",
        endpoint: "38.47.191.253:7777",
    },
];

impl ServerInfo {
    pub fn resolve(&self) -> Option<SocketAddr> {
        match self.endpoint.to_socket_addrs() {
            Ok(mut addrs) => addrs.next(),
            Err(e) => {
                console::err(format!(
                    "Failed to resolve [{}] ({}) -> {}",
                    self.name, self.endpoint, e
                ));
                None
            }
        }
    }
}

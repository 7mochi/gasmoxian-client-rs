use anyhow::{Context, Result};
use rusty_enet as enet;
use std::net::UdpSocket;

/// A wrapper around `rusty_enet` providing the networking layer for the Gasmoxian client.
///
/// Manages a single ENet connection: host creation, peer connection, reliable/unsequenced
/// packet send, event polling, and disconnection.
pub struct EnetClient {
    host: enet::Host<UdpSocket>,
    peer_id: enet::PeerID,
}

impl EnetClient {
    /// Creates a dummy client for use before a real connection is established.
    ///
    /// State functions 0-1 (`LaunchEnterPid`, `LaunchPickServer`) accept it in their
    /// signature but never call methods on it.
    ///
    /// # Panics
    /// Panics if UDP socket binding, ENet host creation, or peer allocation fails.
    /// This should never happen under normal conditions. If it does, the system
    /// has run out of ephemeral ports or similar resource exhaustion.
    pub fn dummy() -> Self {
        let socket =
            UdpSocket::bind("0.0.0.0:0").expect("dummy EnetClient: failed to bind UDP socket");
        let mut host = enet::Host::new(
            socket,
            enet::HostSettings {
                peer_limit: 1,
                channel_limit: 2,
                ..Default::default()
            },
        )
        .expect("dummy EnetClient: failed to create ENet host");
        let addr: std::net::SocketAddr = "127.0.0.1:1"
            .parse()
            .expect("dummy EnetClient: invalid address");
        let peer = host
            .connect(addr, 2, 0)
            .expect("dummy EnetClient: no available peers");
        let peer_id = peer.id();
        EnetClient { host, peer_id }
    }

    /// Connects to a Gasmoxian OnlineCTR server at `addr`.
    ///
    /// Creates a UDP socket, an ENet host with 1 peer slot and 2 channels,
    /// and initiates a connection to the given address.
    ///
    /// # Panics
    /// Panics if the address string cannot be parsed as a socket address
    /// (should never happen since the address is pre-resolved).
    ///
    /// # Errors
    /// Returns an error if socket binding, host creation, or connection initiation fails.
    pub fn new(addr: std::net::SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .with_context(|| format!("failed to bind UDP socket for ENet"))?;
        let mut host = enet::Host::new(
            socket,
            enet::HostSettings {
                peer_limit: 1,
                channel_limit: 2,
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("failed to create ENet host: {:?}", e))?;
        let peer = host
            .connect(addr, 2, 0)
            .map_err(|_| anyhow::anyhow!("no available ENet peers"))?;
        let peer_id = peer.id();
        Ok(EnetClient { host, peer_id })
    }

    /// Sends a ping to keep the connection alive.
    pub fn ping(&mut self) {
        self.host.peer_mut(self.peer_id).ping();
    }

    /// Sends a reliable (guaranteed delivery, in-order) packet on channel 0.
    pub fn send_reliable(&mut self, data: &[u8]) {
        let packet = enet::Packet::reliable(data);
        let _ = self.host.peer_mut(self.peer_id).send(0, &packet);
    }

    /// Sends an unsequenced (best-effort, no ordering) packet on channel 0.
    /// Used for position/state updates where a dropped packet is acceptable.
    pub fn send_unsequenced(&mut self, data: &[u8]) {
        let packet = enet::Packet::unreliable_unsequenced(data);
        let _ = self.host.peer_mut(self.peer_id).send(0, &packet);
    }

    /// Polls the ENet host for incoming events (receive, disconnect, etc.).
    ///
    /// Returns `Ok(None)` if no events are pending.
    ///
    /// # Errors
    /// Returns an error if the underlying socket read/write fails.
    pub fn poll(&mut self) -> Result<Option<enet::Event<'_, UdpSocket>>> {
        Ok(self.host.service()?)
    }

    /// Forcefully disconnects from the server (immediate, no graceful teardown).
    pub fn disconnect_now(&mut self) {
        self.host.peer_mut(self.peer_id).disconnect_now(0);
    }
}

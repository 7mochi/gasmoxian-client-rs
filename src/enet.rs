//! Thin enet wrapper using `rusty_enet`.
//!
//! Provides a blocking handshake (`connect_with_handshake`) with a
//! 3-second timeout, reliable and unsequenced send methods, and
//! a polling interface for receiving events.

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use rusty_enet as enet;

pub struct EnetClient {
    host: enet::Host<UdpSocket>,
    peer_id: enet::PeerID,
}

fn create_host(addr: SocketAddr) -> Result<(enet::Host<UdpSocket>, enet::PeerID)> {
    let socket = UdpSocket::bind("0.0.0.0:0").context("failed to bind UDP socket for ENet")?;
    let mut host = enet::Host::new(
        socket,
        enet::HostSettings {
            peer_limit: 1,
            channel_limit: 2,
            ..Default::default()
        },
    )
    .map_err(|e| anyhow::anyhow!("failed to create ENet host: {:?}", e))?;

    let peer_id = host
        .connect(addr, 2, 0)
        .context("no available ENet peers to establish connection")?
        .id();

    Ok((host, peer_id))
}

impl EnetClient {
    pub fn connect_with_handshake(addr: SocketAddr) -> Result<Self> {
        let (mut host, peer_id) = create_host(addr)?;

        let timeout = Duration::from_secs(3);
        let start = Instant::now();

        // wait up to 3 seconds for the enet connection handshake
        while start.elapsed() < timeout {
            match host.service() {
                Ok(Some(enet::Event::Connect { .. })) => {
                    return Ok(EnetClient { host, peer_id });
                }
                Ok(Some(_)) | Ok(None) => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("ENet handshake error: {:?}", e));
                }
            }
        }

        Err(anyhow::anyhow!("ENet connection timed out after 3s"))
    }

    pub fn ping(&mut self) {
        self.host.peer_mut(self.peer_id).ping();
    }
    pub fn send_reliable(&mut self, data: &[u8]) -> Result<()> {
        let packet = enet::Packet::reliable(data);
        self.host
            .peer_mut(self.peer_id)
            .send(0, &packet)
            .map_err(|e| anyhow::anyhow!("failed to send reliable packet: {:?}", e))
    }

    pub fn send_unsequenced(&mut self, data: &[u8]) -> Result<()> {
        let packet = enet::Packet::unreliable_unsequenced(data);
        self.host
            .peer_mut(self.peer_id)
            .send(0, &packet)
            .map_err(|e| anyhow::anyhow!("failed to send unsequenced packet: {:?}", e))
    }

    pub fn poll(&mut self) -> Result<Option<enet::Event<'_, UdpSocket>>> {
        self.host
            .service()
            .context("failed during ENet host polling")
    }

    pub fn disconnect_now(&mut self) {
        self.host.peer_mut(self.peer_id).disconnect_now(0);
    }
}

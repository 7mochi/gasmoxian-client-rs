use rusty_enet as enet;
use std::net::UdpSocket;

pub struct EnetClient {
    host: enet::Host<UdpSocket>,
    peer_id: enet::PeerID,
}

impl EnetClient {
    pub fn new(addr: std::net::SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let mut host = enet::Host::new(
            socket,
            enet::HostSettings {
                peer_limit: 1,
                channel_limit: 2,
                ..Default::default()
            },
        )?;
        let peer = host.connect(addr, 2, 0)?;
        let peer_id = peer.id();
        Ok(EnetClient { host, peer_id })
    }

    pub fn ping(&mut self) {
        self.host.peer_mut(self.peer_id).ping();
    }

    pub fn send_reliable(&mut self, data: &[u8]) {
        let packet = enet::Packet::reliable(data);
        let _ = self.host.peer_mut(self.peer_id).send(0, &packet);
    }

    pub fn send_unsequenced(&mut self, data: &[u8]) {
        let packet = enet::Packet::unreliable_unsequenced(data);
        let _ = self.host.peer_mut(self.peer_id).send(0, &packet);
    }

    pub fn poll(
        &mut self,
    ) -> Result<Option<enet::Event<'_, UdpSocket>>, Box<dyn std::error::Error>> {
        Ok(self.host.service()?)
    }

    pub fn disconnect_now(&mut self) {
        self.host.peer_mut(self.peer_id).disconnect_now(0);
    }
}

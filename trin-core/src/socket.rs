use log::{debug, info};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

#[cfg(unix)]
use interfaces::{self, Interface};

#[cfg(windows)]
use ipconfig;

// This stun server is part of the testnet infrastructure.
// If you are unable to connect, please create an issue.
const STUN_SERVER: &str = "159.223.0.83:3478";

/// Ping a STUN server on the public network. This does two things:
/// - Creates an externally-addressable UDP port, if you are behind a NAT
/// - Returns the public IP and port that corresponds to your local port
pub fn stun_for_external(local_socket_addr: &SocketAddr) -> Option<SocketAddr> {
    let socket = UdpSocket::bind(local_socket_addr).unwrap();
    info!("Blocking: connecting to STUN server to find public network endpoint");
    let external_addr =
        stunclient::StunClient::new(STUN_SERVER.parse().unwrap()).query_external_address(&socket);

    match external_addr {
        Ok(addr) => {
            debug!("STUN gave us a public address: {:?}", addr);
            Some(addr)
        }
        Err(err) => {
            debug!("Failed to setup STUN traversal: {:?}", err);
            None
        }
    }
}

pub fn default_local_address(port: u16) -> SocketAddr {
    let ip = find_assigned_ip().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    SocketAddr::new(ip, port)
}

#[cfg(unix)]
fn find_assigned_ip() -> Option<IpAddr> {
    let online_nics = Interface::get_all()
        .unwrap_or_default()
        .into_iter()
        .filter(|iface| iface.is_up() && iface.is_running() && !iface.is_loopback());

    for nic in online_nics {
        let ipv4_socket_addr = nic
            .addresses
            .iter()
            .filter(|&addr_group| addr_group.kind == interfaces::Kind::Ipv4)
            .find_map(|addr_group| addr_group.addr);

        if let Some(valid_socket) = ipv4_socket_addr {
            return Some(valid_socket.ip());
        }
        // else, check the next interface
    }
    None
}

#[cfg(windows)]
fn find_assigned_ip() -> Option<IpAddr> {
    let adapters = ipconfig::get_adapters().unwrap_or_default();

    for adapter in adapters.iter() {
        if !adapter.gateways().is_empty() {
            for ip in adapter.ip_addresses().iter() {
                if ip.is_ipv4() {
                    return Some(*ip);
                }
            }
        }
    }
    None
}

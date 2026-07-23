use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent};

use crate::SyncError;

pub const SERVICE_TYPE: &str = "_liauth._tcp.local.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    pub name: String,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
}

pub fn address_priority(address: &IpAddr) -> u8 {
    match address {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            let private = octets[0] == 192 && octets[1] == 168
                || octets[0] == 10
                || octets[0] == 172 && (16..=31).contains(&octets[1]);
            if private {
                0
            } else if v4.is_link_local() {
                2
            } else {
                1
            }
        }
        IpAddr::V6(_) => 3,
    }
}

pub fn discover(timeout: Duration) -> Result<Vec<Peer>, SyncError> {
    let daemon = ServiceDaemon::new().map_err(|e| SyncError::Discovery(e.to_string()))?;
    let browser = daemon
        .browse(SERVICE_TYPE)
        .map_err(|e| SyncError::Discovery(e.to_string()))?;

    let mut peers: HashMap<String, Peer> = HashMap::new();
    let deadline = std::time::Instant::now() + timeout;
    while let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) {
        match browser.recv_timeout(remaining) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                let mut addresses: Vec<IpAddr> = info.get_addresses().iter().copied().collect();
                if addresses.is_empty() {
                    continue;
                }
                addresses.sort_by_key(address_priority);
                let name = info
                    .get_fullname()
                    .split(&format!(".{SERVICE_TYPE}"))
                    .next()
                    .unwrap_or("device")
                    .to_string();
                let entry = peers.entry(info.get_fullname().to_string()).or_insert(Peer {
                    name,
                    addresses: Vec::new(),
                    port: info.get_port(),
                });
                for address in addresses {
                    if !entry.addresses.contains(&address) {
                        entry.addresses.push(address);
                    }
                }
                entry.addresses.sort_by_key(address_priority);
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    let _ = daemon.shutdown();
    Ok(peers.into_values().collect())
}

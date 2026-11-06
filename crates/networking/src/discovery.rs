use serde::{Deserialize, Serialize};
use std::net::IpAddr;

pub const MDNS_SERVICE_TYPE: &str = "_audiobridge._tcp.local.";
pub const DEFAULT_CONTROL_PORT: u16 = 40120;
pub const DEFAULT_MEDIA_PORT: u16 = 40121;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceMetadata {
    pub server_id: String,
    pub server_name: String,
    pub control_port: u16,
    pub media_port: u16,
    pub protocol_version: u8,
    pub requires_pairing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredHost {
    pub id: String,
    pub name: String,
    pub ip: IpAddr,
    pub control_port: u16,
    pub media_port: u16,
    pub is_online: bool,
}

impl DiscoveredHost {
    pub fn new(id: String, name: String, ip: IpAddr, control_port: u16, media_port: u16) -> Self {
        Self {
            id,
            name,
            ip,
            control_port,
            media_port,
            is_online: true,
        }
    }
}

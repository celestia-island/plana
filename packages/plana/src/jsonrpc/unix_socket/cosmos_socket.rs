use std::path::PathBuf;

pub const COSMOS_SOCKET_DIR: &str = "/tmp/entelecheia-unix-socket";
pub const COSMOS_SOCKET_SUFFIX: &str = ".socket";
pub const COSMOS_BRIDGE_SOCKET: &str = "haplotes-bridge.socket";

pub fn cosmos_socket_dir() -> PathBuf {
    std::env::var("COSMOS_SOCKET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(COSMOS_SOCKET_DIR))
}

pub fn cosmos_socket_path(instance_uuid: impl std::fmt::Display) -> PathBuf {
    cosmos_socket_dir().join(format!("{}{}", instance_uuid, COSMOS_SOCKET_SUFFIX))
}

pub fn cosmos_bridge_socket_path() -> PathBuf {
    cosmos_socket_dir().join(COSMOS_BRIDGE_SOCKET)
}

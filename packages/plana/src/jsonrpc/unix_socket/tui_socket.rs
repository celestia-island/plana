use std::path::PathBuf;

const TUI_SOCKET_FILENAME: &str = "entelecheia-tui.sock";

pub fn tui_socket_path() -> PathBuf {
    let dir = super::log_socket::log_socket_dir();
    dir.join(TUI_SOCKET_FILENAME)
}

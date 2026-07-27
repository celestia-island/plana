use bollard::Docker;

use crate::errors::{ContainerError, ContainerResult};

pub fn connect_local() -> ContainerResult<Docker> {
    Docker::connect_with_local_defaults().map_err(|e| ContainerError::Connection(e.to_string()))
}

pub fn connect_socket(socket_path: &str) -> ContainerResult<Docker> {
    Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION)
        .map_err(|e| ContainerError::Connection(e.to_string()))
}

pub fn connect_http(host: &str) -> ContainerResult<Docker> {
    Docker::connect_with_http(host, 120, bollard::API_DEFAULT_VERSION)
        .map_err(|e| ContainerError::Connection(e.to_string()))
}

#[cfg(windows)]
pub fn connect_named_pipe(pipe_name: &str) -> ContainerResult<Docker> {
    Docker::connect_with_named_pipe(pipe_name, 30, bollard::API_DEFAULT_VERSION)
        .map_err(|e| ContainerError::Connection(e.to_string()))
}

pub fn connect_auto() -> ContainerResult<Docker> {
    #[cfg(windows)]
    {
        connect_named_pipe("//./pipe/docker_engine")
    }
    #[cfg(not(windows))]
    {
        connect_local()
    }
}

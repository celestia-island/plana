use std::{io, thread::JoinHandle};

use tracing::{debug, error, info};

pub fn spawn_named<F, T>(name: &str, f: F) -> io::Result<JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    info!(thread = name, "thread spawning");
    std::thread::Builder::new()
        .name(name.to_string())
        .spawn(f)
        .map_err(|e| {
            error!(
                thread = name,
                error = %e,
                "spawn_named: OS thread creation failed (system resource exhaustion)"
            );
            e
        })
}

pub fn block_on_with_handle<F>(handle: &tokio::runtime::Handle, future: F) -> F::Output
where
    F: std::future::Future + Send,
    F::Output: Send,
{
    handle.block_on(future)
}

pub async fn spawn_blocking_named<F, T>(name: &str, f: F) -> tokio::task::JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    info!(blocking_task = name, "spawn_blocking task");
    let name_owned = name.to_string();
    tokio::task::spawn_blocking(move || {
        debug!(blocking_task = %name_owned, "spawn_blocking started");
        let result = f();
        debug!(blocking_task = %name_owned, "spawn_blocking finished");
        result
    })
}

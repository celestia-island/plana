use anyhow::{Result, anyhow};
use futures::future::FutureExt;
use std::panic::AssertUnwindSafe;

use tracing::{error, warn};

pub async fn spawn_guarded<F, T>(name: &str, future: F) -> Option<T>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    match AssertUnwindSafe(future).catch_unwind().await {
        Ok(val) => Some(val),
        Err(panic_payload) => {
            let msg = extract_panic_message(&panic_payload);
            error!(task = %name, panic = %msg, "task panicked — caught by panic_guard");
            None
        }
    }
}

pub async fn catch_async<F, T>(name: &str, future: F) -> Result<T>
where
    F: std::future::Future<Output = T>,
{
    match AssertUnwindSafe(future).catch_unwind().await {
        Ok(val) => Ok(val),
        Err(panic_payload) => {
            let msg = extract_panic_message(&panic_payload);
            warn!(context = %name, panic = %msg, "caught panic in async context");
            Err(anyhow!("Panic in '{}': {}", name, msg))
        }
    }
}

fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    match (
        payload.downcast_ref::<&str>(),
        payload.downcast_ref::<String>(),
    ) {
        (Some(s), _) => s.to_string(),
        (_, Some(s)) => s.clone(),
        _ => "<non-string panic>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[tokio::test]
    async fn test_catch_async_ok() -> anyhow::Result<()> {
        assert_eq!(catch_async("test", async { 42 }).await?, 42);
        Ok(())
    }

    #[tokio::test]
    async fn test_catch_async_panic() -> anyhow::Result<()> {
        let result = catch_async("test", async {
            std::panic::panic_any("boom");
        })
        .await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .context("expected error")?
                .to_string()
                .contains("boom")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_spawn_guarded_ok() -> anyhow::Result<()> {
        let result = spawn_guarded("test", async { 99 }).await;
        assert_eq!(result, Some(99));
        Ok(())
    }

    #[tokio::test]
    async fn test_spawn_guarded_panic() -> anyhow::Result<()> {
        let result = spawn_guarded("test", async {
            std::panic::panic_any("task exploded");
        })
        .await;
        assert_eq!(result, None);
        Ok(())
    }
}

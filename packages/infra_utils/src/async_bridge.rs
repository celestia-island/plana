use anyhow::{Context, Result, anyhow};
use std::future::Future;

pub fn block_on<F>(future: F) -> Result<F::Output>
where
    F: Future + Send,
    F::Output: Send,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            Ok(tokio::task::block_in_place(|| handle.block_on(future)))
        },
        _ => std::thread::scope(|s| {
            let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("failed to create tokio runtime")?;
            let handle = builder
                .spawn_scoped(s, move || rt.block_on(future))
                .context("failed to spawn scoped thread")?;
            let result = handle
                .join()
                .map_err(|e| anyhow!("block_on task panicked: {:?}", e))?;
            Ok(result)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn block_on_outside_runtime() -> anyhow::Result<()> {
        let result = block_on(async { 42 });
        assert_eq!(result?, 42);
        Ok(())
    }

    #[test]
    fn block_on_sequential() -> anyhow::Result<()> {
        let a = block_on(async { 1 });
        let b = block_on(async { 2 });
        assert_eq!((a?, b?), (1, 2));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn block_on_inside_multi_thread_runtime() -> anyhow::Result<()> {
        let result = block_on(async { 42 });
        assert_eq!(result?, 42);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn block_on_nested_multi_thread() -> anyhow::Result<()> {
        let outer = block_on(async {
            let inner = block_on(async { 42 })?;
            Ok::<_, anyhow::Error>(inner + 1)
        });
        assert_eq!(outer??, 43);
        Ok(())
    }

    #[tokio::test]
    async fn block_on_inside_current_thread_runtime() -> anyhow::Result<()> {
        let result = block_on(async { 42 });
        assert_eq!(result?, 42);
        Ok(())
    }

    #[test]
    fn block_on_nested_outside_runtime() -> anyhow::Result<()> {
        let outer = block_on(async {
            let inner = block_on(async { 42 })?;
            Ok::<_, anyhow::Error>(inner + 1)
        });
        assert_eq!(outer??, 43);
        Ok(())
    }

    #[test]
    fn block_on_catches_panics_as_errors() -> anyhow::Result<()> {
        let result = block_on(async { std::panic::panic_any("test panic") });
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.err().context("expected error")?);
        assert!(err_msg.contains("panicked"));
        Ok(())
    }

    #[test]
    fn block_on_with_tokio_timer() -> anyhow::Result<()> {
        let result = block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            99
        });
        assert_eq!(result?, 99);
        Ok(())
    }

    #[test]
    fn block_on_with_tokio_net() -> anyhow::Result<()> {
        let result = block_on(async { tokio::net::TcpListener::bind("127.0.0.1:0").await.is_ok() });
        assert!(result?);
        Ok(())
    }

    #[test]
    fn concurrent_block_on_calls() -> anyhow::Result<()> {
        let results: Vec<_> = (0..4)
            .map(|i| std::thread::spawn(move || block_on(async move { i * 10 })))
            .collect();

        let vals: Vec<_> = results
            .into_iter()
            .map(|h| {
                h.join()
                    .map_err(|e| anyhow!("thread join failed: {:?}", e))
                    .and_then(|r| r.map_err(|e| anyhow!("block_on failed: {:?}", e)))
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(vals, vec![0, 10, 20, 30]);
        Ok(())
    }
}

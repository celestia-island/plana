use anyhow::{Context, Result};
use std::path::PathBuf;
use uuid::Uuid;

use _container::{ops::ContainerOps, types::ContainerCreateParams};

fn data_dir() -> PathBuf {
    _config::UserConfig::config_dir().join("containers")
}

#[tokio::test]
#[ignore = "requires root: sudo cargo test -p _container_runtime --test integration -- --ignored"]
async fn test_full_lifecycle() -> Result<()> {
    let mgr = _container_runtime::YoukiManager::new(&data_dir()).context("create YoukiManager")?;
    mgr.initialize().await.context("initialize")?;

    let params = ContainerCreateParams::simple("test-lifecycle", "host");

    let info = mgr.create(&params).await.context("create")?;
    eprintln!("[CREATE] id={} status={:?}", info.id, info.status);

    mgr.start(&info.id).await.context("start")?;
    eprintln!("[START] ok");

    let running = mgr.is_running(&info.id).await.context("is_running")?;
    assert!(running, "container should be running");
    eprintln!("[CHECK] running={}", running);

    mgr.stop(&info.id).await.context("stop")?;
    eprintln!("[STOP] ok");

    mgr.remove(&info.id, true).await.context("remove")?;
    eprintln!("[REMOVE] ok");
    Ok(())
}

#[tokio::test]
#[ignore = "requires root: sudo cargo test -p _container_runtime --test integration -- --ignored"]
async fn test_host_rootfs_overlay() -> Result<()> {
    let mgr = _container_runtime::YoukiManager::new(&data_dir()).context("create YoukiManager")?;
    mgr.initialize().await.context("initialize")?;

    let rootfs_mgr = _container_runtime::RootfsManager::new(&data_dir());
    let cid = format!("test-overlay-{}", Uuid::now_v7().as_simple());

    let merged = rootfs_mgr
        .prepare_container_rootfs("host", &cid)
        .await
        .context("prepare overlay rootfs")?;
    eprintln!("[OVERLAY] merged={}", merged.display());

    assert!(
        merged.join("bin").exists(),
        "/bin should exist in merged rootfs"
    );
    assert!(
        merged.join("usr").exists(),
        "/usr should exist in merged rootfs"
    );
    let proc_empty = if merged.join("proc").is_dir() {
        let mut entries = tokio::fs::read_dir(merged.join("proc")).await?;
        entries.next_entry().await?.is_none()
    } else {
        true
    };
    assert!(proc_empty, "/proc should be empty");

    rootfs_mgr
        .cleanup_container_rootfs(&cid)
        .await
        .context("cleanup")?;
    eprintln!("[CLEANUP] ok");
    Ok(())
}

#[tokio::test]
#[ignore = "requires root: sudo cargo test -p _container_runtime --test integration -- --ignored"]
async fn test_list_and_inspect() -> Result<()> {
    let mgr = _container_runtime::YoukiManager::new(&data_dir()).context("create YoukiManager")?;
    mgr.initialize().await.context("initialize")?;

    let params = ContainerCreateParams::simple("test-list", "host");
    let info = mgr.create(&params).await.context("create")?;

    let list = mgr.list().await.context("list")?;
    assert!(
        list.iter().any(|c| c.id == info.id),
        "created container should appear in list"
    );

    let detail = mgr.inspect(&info.id).await.context("inspect")?;
    assert_eq!(detail.info.id, info.id);

    mgr.remove(&info.id, true).await.context("cleanup")?;
    Ok(())
}

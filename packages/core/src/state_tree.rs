use anyhow::Result;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::watch;

pub struct StateTree<T> {
    inner: Arc<StateTreeInner<T>>,
}

struct StateTreeInner<T> {
    state: RwLock<T>,
    version: AtomicU64,
    notify_tx: watch::Sender<u64>,
}

impl<T> StateTree<T> {
    pub fn new(initial: T) -> Self {
        let (notify_tx, _) = watch::channel(0);
        Self {
            inner: Arc::new(StateTreeInner {
                state: RwLock::new(initial),
                version: AtomicU64::new(0),
                notify_tx,
            }),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.state.read()
    }

    pub fn write(&self) -> StateTreeWriteGuard<'_, T> {
        StateTreeWriteGuard {
            guard: self.inner.state.write(),
            version: &self.inner.version,
            notify_tx: &self.inner.notify_tx,
            bumped: false,
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.inner.notify_tx.subscribe()
    }

    pub fn version(&self) -> u64 {
        self.inner.version.load(Ordering::Relaxed)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn get_mut(&mut self) -> Result<&mut T, &'static str> {
        match Arc::get_mut(&mut self.inner) {
            Some(inner) => Ok(inner.state.get_mut()),
            None => Err("StateTree::get_mut failed: Arc is shared (other clones exist)"),
        }
    }
}

impl<T> Clone for StateTree<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Default> Default for StateTree<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub struct StateTreeWriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, T>,
    version: &'a AtomicU64,
    notify_tx: &'a watch::Sender<u64>,
    bumped: bool,
}

impl<'a, T> StateTreeWriteGuard<'a, T> {
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    pub fn bump(&mut self) {
        if !self.bumped {
            let next = self.version.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = self.notify_tx.send(next);
            self.bumped = true;
        }
    }
}

impl<'a, T> std::ops::Deref for StateTreeWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> std::ops::DerefMut for StateTreeWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a, T> Drop for StateTreeWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.bump();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc as StdArc;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_basic_read_write() -> Result<()> {
        let tree = StateTree::new(0u64);
        assert_eq!(*tree.read(), 0);
        assert_eq!(tree.version(), 0);

        {
            let mut w = tree.write();
            *w = 42;
        }

        assert_eq!(*tree.read(), 42);
        assert_eq!(tree.version(), 1);
        Ok(())
    }

    #[test]
    fn test_multiple_writes_bump_version() -> Result<()> {
        let tree = StateTree::new(String::new());
        assert_eq!(tree.version(), 0);

        {
            let mut w = tree.write();
            w.push_str("hello");
        }
        assert_eq!(tree.version(), 1);

        {
            let mut w = tree.write();
            w.push_str(" world");
        }
        assert_eq!(tree.version(), 2);

        assert_eq!(&*tree.read(), "hello world");
        Ok(())
    }

    #[test]
    fn test_watch_notification() -> Result<()> {
        let tree = StateTree::new(0u64);
        let rx = tree.subscribe();
        assert_eq!(*rx.borrow(), 0);
        assert!(!rx.has_changed()?);

        {
            let mut w = tree.write();
            *w = 10;
        }

        assert!(rx.has_changed()?);
        assert_eq!(*rx.borrow(), 1);
        Ok(())
    }

    #[test]
    fn test_explicit_bump_only_once() -> Result<()> {
        let tree = StateTree::new(0u64);
        {
            let mut w = tree.write();
            *w = 1;
            w.bump();
            assert_eq!(w.version(), 1);
            w.bump();
            assert_eq!(w.version(), 1);
        }
        assert_eq!(tree.version(), 1);
        Ok(())
    }

    #[test]
    fn test_clone_shares_state() -> Result<()> {
        let tree = StateTree::new(vec![1, 2, 3]);
        let tree2 = tree.clone();
        assert!(tree.ptr_eq(&tree2));

        {
            let mut w = tree2.write();
            w.push(4);
        }

        assert_eq!(*tree.read(), vec![1, 2, 3, 4]);
        assert_eq!(tree.version(), tree2.version());
        Ok(())
    }

    #[test]
    fn test_multiple_subscribers() -> Result<()> {
        let tree = StateTree::new(0u64);
        let rx1 = tree.subscribe();
        let rx2 = tree.subscribe();

        {
            let mut w = tree.write();
            *w = 99;
        }

        assert!(rx1.has_changed()?);
        assert!(rx2.has_changed()?);
        assert_eq!(*rx1.borrow(), 1);
        assert_eq!(*rx2.borrow(), 1);
        Ok(())
    }

    #[test]
    fn test_version_monotonic() -> Result<()> {
        let tree = StateTree::new(0u64);
        let mut prev = tree.version();
        for i in 1..=100u64 {
            {
                let mut w = tree.write();
                *w = i;
            }
            let cur = tree.version();
            assert!(cur > prev, "version must be monotonic: {} <= {}", cur, prev);
            prev = cur;
        }
        Ok(())
    }

    #[test]
    fn test_concurrent_writers() -> Result<()> {
        let tree = StdArc::new(StateTree::new(0u64));
        let writers = StdArc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();

        for _ in 0..4 {
            let tree = StdArc::clone(&tree);
            let writers = StdArc::clone(&writers);
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    let mut w = tree.write();
                    *w += 1;
                    drop(w);
                    writers.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }

        for h in handles {
            h.join()
                .map_err(|e| anyhow::anyhow!("thread panicked: {:?}", e))?;
        }

        assert_eq!(writers.load(Ordering::Relaxed), 400);
        assert_eq!(*tree.read(), 400);
        assert_eq!(tree.version(), 400);
        Ok(())
    }

    #[test]
    fn test_subscriber_catches_up() -> Result<()> {
        let tree = StateTree::new(0u64);
        let rx = tree.subscribe();

        for i in 1..=5 {
            let mut w = tree.write();
            *w = i;
        }

        assert!(rx.has_changed()?);
        assert_eq!(*rx.borrow(), 5);
        assert_eq!(*tree.read(), 5);
        Ok(())
    }

    #[test]
    fn test_empty_write_still_bumps() -> Result<()> {
        let tree = StateTree::new(42u64);
        assert_eq!(tree.version(), 0);
        {
            let _ = tree.write();
        }
        assert_eq!(tree.version(), 1);
        assert_eq!(*tree.read(), 42);
        Ok(())
    }

    #[test]
    fn test_default() -> Result<()> {
        let tree: StateTree<u32> = StateTree::default();
        assert_eq!(*tree.read(), 0);
        assert_eq!(tree.version(), 0);
        Ok(())
    }
}

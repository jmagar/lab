//! Per-host advisory lock registry.
//!
//! Concurrent deploy runs from the same process must not touch the same host
//! simultaneously. The registry keeps one `tokio::sync::Mutex` per alias;
//! acquires wait up to a caller-chosen timeout, after which they return
//! `DeployError::Conflict`.

use dashmap::DashMap;
use lab_apis::deploy::DeployError;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Advisory lock registry keyed by SSH alias.
#[derive(Default)]
pub struct HostLockRegistry {
    inner: DashMap<String, Arc<Mutex<()>>>,
}

impl HostLockRegistry {
    /// Acquire an advisory lock on `host`, waiting up to `timeout` before
    /// surfacing `Conflict`.
    pub async fn acquire(
        &self,
        host: &str,
        timeout: std::time::Duration,
    ) -> Result<OwnedMutexGuard<()>, DeployError> {
        let mutex = self
            .inner
            .entry(host.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        match tokio::time::timeout(timeout, mutex.lock_owned()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(DeployError::Conflict {
                host: host.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn first_lock_on_host_succeeds() {
        let reg = HostLockRegistry::default();
        let _g = reg
            .acquire("mini1", std::time::Duration::from_millis(50))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn concurrent_lock_on_same_host_returns_conflict() {
        let reg = Arc::new(HostLockRegistry::default());
        let reg2 = reg.clone();
        let _held = reg
            .acquire("mini1", std::time::Duration::from_millis(50))
            .await
            .unwrap();
        let err = reg2
            .acquire("mini1", std::time::Duration::from_millis(25))
            .await
            .unwrap_err();
        assert_eq!(err.kind(), "conflict");
    }

    #[tokio::test]
    async fn different_hosts_do_not_conflict() {
        let reg = Arc::new(HostLockRegistry::default());
        let _a = reg
            .acquire("mini1", std::time::Duration::from_millis(50))
            .await
            .unwrap();
        let _b = reg
            .acquire("mini2", std::time::Duration::from_millis(50))
            .await
            .unwrap();
    }
}

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrKey {
    pub owner: String,
    pub repo: String,
    pub pr: u64,
}

type BoxFut = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;
type FlushFn = Arc<dyn Fn(PrKey, u32) -> BoxFut + Send + Sync>;

struct Entry {
    count: u32,
    generation: u64,
    handle: JoinHandle<()>,
}

pub struct Debouncer {
    window: Duration,
    flush: FlushFn,
    state: Arc<Mutex<HashMap<PrKey, Entry>>>,
}

impl Debouncer {
    pub fn new<F, Fut>(window: Duration, flush: F) -> Self
    where
        F: Fn(PrKey, u32) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let flush: FlushFn = Arc::new(move |k, n| Box::pin(flush(k, n)));
        // std::sync::Mutex is correct here — we never hold the guard across an .await.
        Self {
            window,
            flush,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn hit(&self, key: PrKey) {
        let generation;
        let count;
        let old_handle;
        {
            let mut map = self.state.lock().unwrap();
            let entry = map.remove(&key);
            let (prev_count, _prev_gen, prev_handle) = match entry {
                Some(e) => (e.count, e.generation, Some(e.handle)),
                None => (0, 0, None),
            };
            count = prev_count + 1;
            generation = fresh_generation();
            old_handle = prev_handle;
        }
        if let Some(h) = old_handle {
            h.abort();
        }
        let handle = self.spawn_timer(key.clone(), generation);
        let mut map = self.state.lock().unwrap();
        map.insert(
            key,
            Entry {
                count,
                generation,
                handle,
            },
        );
    }

    fn spawn_timer(&self, key: PrKey, generation: u64) -> JoinHandle<()> {
        let window = self.window;
        let flush = self.flush.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(window).await;
            // Generation check: if a newer hit replaced us, the newer entry has a different
            // generation. Only the matching generation should flush, preventing double-flush
            // when abort races with timer wakeup.
            let n = {
                let mut map = state.lock().unwrap();
                match map.get(&key) {
                    Some(e) if e.generation == generation => {
                        let n = e.count;
                        map.remove(&key);
                        n
                    }
                    _ => return, // stale or superseded
                }
            };
            if let Err(e) = (flush)(key, n).await {
                tracing::error!(target: "gh_webhook::debounce", error = %e, "flush failed");
            }
        })
    }

    pub async fn drain(&self) {
        let entries: Vec<(PrKey, u32, JoinHandle<()>)> = {
            let mut map = self.state.lock().unwrap();
            map.drain().map(|(k, e)| (k, e.count, e.handle)).collect()
        };
        for (key, count, handle) in entries {
            handle.abort();
            if let Err(e) = (self.flush)(key, count).await {
                tracing::error!(target: "gh_webhook::debounce", error = %e, "drain flush failed");
            }
        }
    }
}

fn fresh_generation() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static GEN: AtomicU64 = AtomicU64::new(1);
    GEN.fetch_add(1, Ordering::Relaxed)
}

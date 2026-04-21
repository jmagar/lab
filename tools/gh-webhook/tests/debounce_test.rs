use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use gh_webhook::debounce::{Debouncer, PrKey};

#[tokio::test]
async fn coalesces_burst_to_single_flush() {
    let calls = Arc::new(AtomicU32::new(0));
    let counter = Arc::new(AtomicU32::new(0));
    let calls_c = calls.clone();
    let counter_c = counter.clone();
    let d = Debouncer::new(Duration::from_millis(100), move |_key, n| {
        let calls = calls_c.clone();
        let counter = counter_c.clone();
        async move {
            calls.fetch_add(1, Ordering::SeqCst);
            counter.fetch_add(n, Ordering::SeqCst);
            Ok::<_, anyhow::Error>(())
        }
    });
    let key = PrKey {
        owner: "o".into(),
        repo: "r".into(),
        pr: 1,
    };
    for _ in 0..5 {
        d.hit(key.clone()).await;
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "should have flushed exactly once"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        5,
        "should have flushed with count=5"
    );
}

#[tokio::test]
async fn drain_forces_flush_immediately() {
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();
    let d = Debouncer::new(Duration::from_secs(60), move |_key, n| {
        let c = c.clone();
        async move {
            c.fetch_add(n, Ordering::SeqCst);
            Ok::<_, anyhow::Error>(())
        }
    });
    let key = PrKey {
        owner: "o".into(),
        repo: "r".into(),
        pr: 1,
    };
    d.hit(key.clone()).await;
    d.hit(key).await;
    d.drain().await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "drain should flush with count=2 before debounce elapses"
    );
}

#[tokio::test]
async fn independent_prs_flush_independently() {
    let seen: Arc<Mutex<Vec<(PrKey, u32)>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_c = seen.clone();
    let d = Debouncer::new(Duration::from_millis(100), move |key, n| {
        let seen = seen_c.clone();
        async move {
            seen.lock().unwrap().push((key, n));
            Ok::<_, anyhow::Error>(())
        }
    });
    let k1 = PrKey {
        owner: "o".into(),
        repo: "r".into(),
        pr: 1,
    };
    let k2 = PrKey {
        owner: "o".into(),
        repo: "r".into(),
        pr: 2,
    };
    d.hit(k1.clone()).await;
    d.hit(k2.clone()).await;
    d.hit(k1.clone()).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let mut recorded = seen.lock().unwrap().clone();
    recorded.sort_by_key(|(k, _)| k.pr);
    assert_eq!(recorded.len(), 2, "both PRs should flush");
    assert_eq!(recorded[0], (k1, 2));
    assert_eq!(recorded[1], (k2, 1));
}

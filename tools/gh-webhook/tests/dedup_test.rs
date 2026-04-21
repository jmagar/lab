use gh_webhook::dedup::DedupCache;

#[test]
fn fresh_id_returns_false_and_inserts() {
    let mut c = DedupCache::new(4);
    assert!(!c.seen("abc"));
    // It is now recorded; a second call must report it as seen.
    assert!(c.seen("abc"));
}

#[test]
fn repeat_id_returns_true() {
    let mut c = DedupCache::new(4);
    assert!(!c.seen("abc"));
    assert!(c.seen("abc"));
    assert!(c.seen("abc"));
}

#[test]
fn fifo_eviction_oldest_first() {
    let mut c = DedupCache::new(3);
    assert!(!c.seen("A"));
    assert!(!c.seen("B"));
    assert!(!c.seen("C"));
    // Inserting D evicts A (oldest, FIFO).
    assert!(!c.seen("D"));
    // B, C, D are still present.
    assert!(c.seen("B"));
    assert!(c.seen("C"));
    assert!(c.seen("D"));
    // A was evicted, so it is fresh again.
    assert!(!c.seen("A"));
}

//! Bounded FIFO delivery-id dedup cache.
//!
//! Tracks recently seen GitHub `X-GitHub-Delivery` ids so that duplicate
//! webhook deliveries (e.g. from automatic retries) can be rejected without
//! re-processing. The cache has a fixed capacity; once full, inserting a new
//! id evicts the oldest one (FIFO) — both from the ordering deque and from
//! the membership set.
//!
//! The cache is intentionally *not* internally synchronized: `seen` takes
//! `&mut self`. Callers that need shared access should wrap the cache in a
//! `Mutex` or equivalent.

use std::collections::{HashSet, VecDeque};

/// Bounded FIFO cache of delivery ids.
pub struct DedupCache {
    capacity: usize,
    order: VecDeque<String>,
    set: HashSet<String>,
}

impl DedupCache {
    /// Create a new cache with the given capacity.
    ///
    /// A capacity of zero means every `seen` call reports "fresh" but records
    /// nothing (the entry is immediately evicted).
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            order: VecDeque::with_capacity(capacity),
            set: HashSet::with_capacity(capacity),
        }
    }

    /// Returns `true` if `delivery_id` has already been seen.
    ///
    /// If the id is new, it is inserted and `false` is returned. When the
    /// cache is full, the oldest id is evicted (popped from the front of the
    /// deque and removed from the set) before the new id is inserted.
    pub fn seen(&mut self, delivery_id: &str) -> bool {
        if self.set.contains(delivery_id) {
            return true;
        }

        // New id — enforce capacity via FIFO eviction before inserting.
        while self.order.len() >= self.capacity {
            match self.order.pop_front() {
                Some(old) => {
                    self.set.remove(&old);
                }
                None => break, // capacity == 0 with empty deque
            }
        }

        if self.capacity > 0 {
            let owned = delivery_id.to_string();
            self.order.push_back(owned.clone());
            self.set.insert(owned);
        }

        false
    }
}

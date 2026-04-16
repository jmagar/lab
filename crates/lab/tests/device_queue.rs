use lab::device::queue::{DeviceOutboundQueue, QueuedEnvelope};

#[tokio::test]
async fn queue_persists_and_reloads_entries() {
    let temp = tempfile::tempdir().unwrap();
    let queue = DeviceOutboundQueue::open(temp.path().join("queue.jsonl"))
        .await
        .unwrap();

    queue
        .push(QueuedEnvelope::status(
            serde_json::json!({"device_id":"tootie"}),
        ))
        .await
        .unwrap();
    drop(queue);

    let reopened = DeviceOutboundQueue::open(temp.path().join("queue.jsonl"))
        .await
        .unwrap();
    let drained = reopened.drain_batch(10).await.unwrap();
    assert_eq!(drained.len(), 1);
}

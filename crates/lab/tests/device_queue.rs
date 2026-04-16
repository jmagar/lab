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
    assert_eq!(drained[0].payload["device_id"], "tootie");
}

#[tokio::test]
async fn queue_ack_uses_latest_on_disk_entries_when_multiple_handles_share_a_path() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("queue.jsonl");

    let first = DeviceOutboundQueue::open(path.clone()).await.unwrap();
    first
        .push(QueuedEnvelope::status(
            serde_json::json!({"device_id":"first"}),
        ))
        .await
        .unwrap();

    let second = DeviceOutboundQueue::open(path.clone()).await.unwrap();
    second
        .push(QueuedEnvelope::status(
            serde_json::json!({"device_id":"second"}),
        ))
        .await
        .unwrap();

    first.ack_drained(1).await.unwrap();

    let reopened = DeviceOutboundQueue::open(path).await.unwrap();
    let drained = reopened.drain_batch(10).await.unwrap();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].payload["device_id"], "second");
}

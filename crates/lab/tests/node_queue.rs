use lab::node::queue::{NodeOutboundQueue, QueuedEnvelope};

#[tokio::test]
async fn queue_persists_and_reloads_entries() {
    let temp = tempfile::tempdir().unwrap();
    let queue = NodeOutboundQueue::open(temp.path().join("queue.jsonl"))
        .await
        .unwrap();

    queue
        .push(QueuedEnvelope::status(
            serde_json::json!({"node_id":"tootie"}),
        ))
        .await
        .unwrap();
    drop(queue);

    let reopened = NodeOutboundQueue::open(temp.path().join("queue.jsonl"))
        .await
        .unwrap();
    let drained = reopened.drain_batch(10).await.unwrap();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].payload["node_id"], "tootie");
}

#[tokio::test]
async fn queue_ack_uses_latest_on_disk_entries_when_multiple_handles_share_a_path() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("queue.jsonl");

    let first = NodeOutboundQueue::open(path.clone()).await.unwrap();
    first
        .push(QueuedEnvelope::status(
            serde_json::json!({"node_id":"first"}),
        ))
        .await
        .unwrap();

    let second = NodeOutboundQueue::open(path.clone()).await.unwrap();
    second
        .push(QueuedEnvelope::status(
            serde_json::json!({"node_id":"second"}),
        ))
        .await
        .unwrap();

    first.ack_drained(1).await.unwrap();

    let reopened = NodeOutboundQueue::open(path).await.unwrap();
    let drained = reopened.drain_batch(10).await.unwrap();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].payload["node_id"], "second");
}

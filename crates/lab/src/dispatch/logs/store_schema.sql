CREATE TABLE IF NOT EXISTS log_events (
    event_id          TEXT PRIMARY KEY,
    ts                INTEGER NOT NULL,
    level             TEXT NOT NULL,
    subsystem         TEXT NOT NULL,
    surface           TEXT NOT NULL,
    action            TEXT,
    message           TEXT NOT NULL,
    request_id        TEXT,
    session_id        TEXT,
    correlation_id    TEXT,
    trace_id          TEXT,
    span_id           TEXT,
    instance          TEXT,
    auth_flow         TEXT,
    outcome_kind      TEXT,
    fields_json       TEXT NOT NULL DEFAULT '{}',
    source_kind       TEXT,
    source_node_id    TEXT,
    source_device_id  TEXT,
    ingest_path       TEXT,
    upstream_event_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_log_events_ts         ON log_events(ts DESC);
CREATE INDEX IF NOT EXISTS idx_log_events_level_ts   ON log_events(level, ts DESC);
CREATE INDEX IF NOT EXISTS idx_log_events_subsys_ts  ON log_events(subsystem, ts DESC);
CREATE INDEX IF NOT EXISTS idx_log_events_request_id ON log_events(request_id);
CREATE INDEX IF NOT EXISTS idx_log_events_session_id    ON log_events(session_id);
CREATE INDEX IF NOT EXISTS idx_log_events_source_node   ON log_events(source_node_id, ts DESC);
CREATE INDEX IF NOT EXISTS idx_log_events_source_kind   ON log_events(source_kind, ts DESC);

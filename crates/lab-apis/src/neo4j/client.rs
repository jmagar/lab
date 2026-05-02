//! Neo4j Bolt client.

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use neo4rs::{BoltType, ConfigBuilder, Graph, query};
use serde_json::{Map, Value};
use tokio::time::timeout;

use crate::core::error::ApiError;

use super::error::Neo4jError;
use super::types::{CypherResponse, MAX_ROWS, Statement, TxMode, WriteResponse};

const QUERY_TIMEOUT: Duration = Duration::from_secs(30);

/// Redacted Neo4j connection URI for diagnostics.
#[derive(Clone, PartialEq, Eq)]
pub struct SanitizedUri(String);

impl SanitizedUri {
    pub fn parse(raw: &str) -> Result<Self, Neo4jError> {
        let parsed = url::Url::parse(raw)
            .map_err(|e| Neo4jError::InvalidParam(format!("invalid NEO4J_URL: {e}")))?;
        match parsed.scheme() {
            "neo4j" | "neo4j+s" | "bolt" | "bolt+s" => {}
            other => return Err(Neo4jError::InsecureScheme(other.to_string())),
        }

        let mut redacted = parsed.clone();
        if !redacted.username().is_empty() {
            let _ = redacted.set_username("[redacted]");
        }
        if redacted.password().is_some() {
            let _ = redacted.set_password(Some("[redacted]"));
        }
        Ok(Self(redacted.to_string()))
    }
}

impl fmt::Display for SanitizedUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for SanitizedUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Neo4j connection options.
#[derive(Debug, Clone)]
pub struct Neo4jConfig {
    pub url: String,
    pub user: String,
    pub password: String,
    pub database: Option<String>,
    pub pool_size: usize,
    pub ca_cert_path: Option<PathBuf>,
}

impl Neo4jConfig {
    pub fn validate(&self) -> Result<SanitizedUri, Neo4jError> {
        if self.user.trim().is_empty() {
            return Err(Neo4jError::InvalidParam(
                "NEO4J_USER must not be empty".into(),
            ));
        }
        if self.password.is_empty() {
            return Err(Neo4jError::InvalidParam(
                "NEO4J_PASSWORD must not be empty".into(),
            ));
        }
        if self.pool_size == 0 {
            return Err(Neo4jError::InvalidParam(
                "NEO4J_POOL_SIZE must be greater than zero".into(),
            ));
        }
        SanitizedUri::parse(&self.url)
    }
}

/// Client for a Neo4j database over Bolt.
#[derive(Clone)]
pub struct Neo4jClient {
    graph: Graph,
    sanitized_uri: SanitizedUri,
}

impl Neo4jClient {
    /// Build a Neo4j client from explicit config.
    pub fn new(config: Neo4jConfig) -> Result<Self, Neo4jError> {
        let sanitized_uri = config.validate()?;
        let mut builder = ConfigBuilder::default()
            .uri(config.url)
            .user(config.user)
            .password(config.password)
            .max_connections(config.pool_size);
        if let Some(database) = config.database.filter(|db| !db.trim().is_empty()) {
            builder = builder.db(database);
        }
        if let Some(path) = config.ca_cert_path {
            builder = builder.with_client_certificate(path);
        }
        let config = builder
            .build()
            .map_err(|e| ApiError::Internal(format!("Neo4j config failed: {e}")))?;
        let graph = Graph::connect(config)
            .map_err(|e| ApiError::Network(format!("Neo4j connect {sanitized_uri}: {e}")))?;
        Ok(Self {
            graph,
            sanitized_uri,
        })
    }

    /// Execute a read-only Cypher statement.
    pub async fn cypher_read(
        &self,
        statement: &str,
        parameters: Map<String, Value>,
    ) -> Result<CypherResponse, Neo4jError> {
        validate_statement(statement)?;
        let query = build_query(statement, parameters)?;
        let mut stream = timeout(QUERY_TIMEOUT, self.graph.execute_read(query))
            .await
            .map_err(|_| Neo4jError::Timeout(QUERY_TIMEOUT.as_secs()))?
            .map_err(map_neo4j_error)?;
        collect_rows(&mut stream).await
    }

    /// Execute a write Cypher statement and discard the stream.
    pub async fn cypher_write(
        &self,
        statement: &str,
        parameters: Map<String, Value>,
    ) -> Result<WriteResponse, Neo4jError> {
        validate_statement(statement)?;
        let query = build_query(statement, parameters)?;
        timeout(QUERY_TIMEOUT, self.graph.run(query))
            .await
            .map_err(|_| Neo4jError::Timeout(QUERY_TIMEOUT.as_secs()))?
            .map_err(map_neo4j_error)?;
        Ok(WriteResponse {
            ok: true,
            statements: 1,
        })
    }

    pub async fn labels(&self) -> Result<CypherResponse, Neo4jError> {
        self.cypher_read("SHOW LABELS", Map::new()).await
    }

    pub async fn relationships(&self) -> Result<CypherResponse, Neo4jError> {
        self.cypher_read("SHOW RELATIONSHIP TYPES", Map::new())
            .await
    }

    pub async fn constraints(&self) -> Result<CypherResponse, Neo4jError> {
        self.cypher_read("SHOW CONSTRAINTS", Map::new()).await
    }

    pub async fn indexes(&self) -> Result<CypherResponse, Neo4jError> {
        self.cypher_read("SHOW INDEXES", Map::new()).await
    }

    pub async fn db_list(&self) -> Result<CypherResponse, Neo4jError> {
        self.cypher_read("SHOW DATABASES", Map::new()).await
    }

    pub async fn db_info(&self, database: Option<&str>) -> Result<CypherResponse, Neo4jError> {
        match database {
            Some(database) if !database.trim().is_empty() => {
                self.cypher_read(
                    &format!("SHOW DATABASE `{}`", escape_name(database)),
                    Map::new(),
                )
                .await
            }
            _ => self.cypher_read("SHOW DATABASE", Map::new()).await,
        }
    }

    pub async fn server_status(&self) -> Result<CypherResponse, Neo4jError> {
        self.cypher_read("CALL dbms.components()", Map::new()).await
    }

    pub async fn txn_run(
        &self,
        statements: Vec<Statement>,
        mode: TxMode,
    ) -> Result<Value, Neo4jError> {
        if statements.is_empty() {
            return Err(Neo4jError::InvalidParam(
                "statements must not be empty".into(),
            ));
        }
        match mode {
            TxMode::R => {
                let mut responses = Vec::with_capacity(statements.len());
                for statement in statements {
                    responses.push(
                        serde_json::to_value(
                            self.cypher_read(&statement.statement, statement.parameters)
                                .await?,
                        )
                        .map_err(|e| ApiError::Decode(e.to_string()))?,
                    );
                }
                Ok(serde_json::json!({ "mode": "r", "responses": responses }))
            }
            TxMode::W => {
                let mut txn = timeout(QUERY_TIMEOUT, self.graph.start_txn())
                    .await
                    .map_err(|_| Neo4jError::Timeout(QUERY_TIMEOUT.as_secs()))?
                    .map_err(map_neo4j_error)?;
                for statement in &statements {
                    validate_statement(&statement.statement)?;
                    let query = build_query(&statement.statement, statement.parameters.clone())?;
                    if timeout(QUERY_TIMEOUT, txn.run(query)).await.is_err() {
                        let _rollback = txn.rollback().await;
                        return Err(Neo4jError::Timeout(QUERY_TIMEOUT.as_secs()));
                    }
                }
                timeout(QUERY_TIMEOUT, txn.commit())
                    .await
                    .map_err(|_| Neo4jError::Timeout(QUERY_TIMEOUT.as_secs()))?
                    .map_err(map_neo4j_error)?;
                Ok(serde_json::json!({ "mode": "w", "ok": true, "statements": statements.len() }))
            }
        }
    }

    #[must_use]
    pub fn sanitized_uri(&self) -> &SanitizedUri {
        &self.sanitized_uri
    }
}

fn validate_statement(statement: &str) -> Result<(), Neo4jError> {
    if statement.trim().is_empty() {
        return Err(Neo4jError::InvalidParam(
            "statement must not be empty".into(),
        ));
    }
    Ok(())
}

fn build_query(
    statement: &str,
    parameters: Map<String, Value>,
) -> Result<neo4rs::Query, Neo4jError> {
    let mut q = query(statement);
    for (key, value) in parameters {
        let bolt: BoltType = value.try_into().map_err(|e| {
            Neo4jError::InvalidParam(format!("parameter `{key}` is not Bolt-compatible: {e}"))
        })?;
        q = q.param(&key, bolt);
    }
    Ok(q)
}

async fn collect_rows(
    stream: &mut neo4rs::DetachedRowStream,
) -> Result<CypherResponse, Neo4jError> {
    let mut rows = Vec::new();
    let mut truncated = false;
    loop {
        let next = timeout(QUERY_TIMEOUT, stream.next())
            .await
            .map_err(|_| Neo4jError::Timeout(QUERY_TIMEOUT.as_secs()))?
            .map_err(map_neo4j_error)?;
        let Some(row) = next else {
            break;
        };
        if rows.len() >= MAX_ROWS {
            truncated = true;
            continue;
        }
        let mut object = Map::new();
        for key in row.keys() {
            let key = key.value.clone();
            let value = row
                .get::<Value>(&key)
                .map_err(|e| ApiError::Decode(format!("Neo4j row field `{key}`: {e}")))?;
            object.insert(key, value);
        }
        rows.push(object);
    }
    Ok(CypherResponse {
        row_count: rows.len(),
        rows,
        truncated,
    })
}

fn map_neo4j_error(err: neo4rs::Error) -> Neo4jError {
    ApiError::Server {
        status: 500,
        body: err.to_string(),
    }
    .into()
}

fn escape_name(name: &str) -> String {
    name.replace('`', "``")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_uri_redacts_credentials() {
        let uri = SanitizedUri::parse("neo4j://user:secret@localhost:7687").unwrap();
        assert_eq!(
            uri.to_string(),
            "neo4j://%5Bredacted%5D:%5Bredacted%5D@localhost:7687"
        );
        assert!(!format!("{uri:?}").contains("secret"));
    }

    #[test]
    fn rejects_insecure_self_signed_scheme() {
        let err = SanitizedUri::parse("neo4j+ssc://localhost:7687").unwrap_err();
        assert!(matches!(err, Neo4jError::InsecureScheme(_)));
    }

    #[test]
    fn converts_json_params_to_bolt_query() {
        let params = serde_json::json!({"name": "Ada", "age": 42})
            .as_object()
            .cloned()
            .unwrap();
        build_query("RETURN $name", params).unwrap();
    }
}

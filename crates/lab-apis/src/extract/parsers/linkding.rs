use std::io::Write;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tempfile::NamedTempFile;

use crate::extract::error::ExtractError;
use crate::extract::parsers::Parser;
use crate::extract::types::ServiceCreds;

pub struct LinkdingParser;

impl Parser for LinkdingParser {
    fn name(&self) -> &'static str {
        "linkding"
    }

    fn config_path(&self, appdata_root: &Path) -> PathBuf {
        appdata_root.join("linkding").join("db.sqlite3")
    }

    fn parse(&self, contents: &[u8]) -> Result<ServiceCreds, ExtractError> {
        let database = write_temp_sqlite(contents)?;
        parse_linkding_database(database.path())
    }
}

fn write_temp_sqlite(contents: &[u8]) -> Result<NamedTempFile, ExtractError> {
    let mut file = NamedTempFile::new().map_err(|source| ExtractError::Io {
        path: PathBuf::from("<tempfile>"),
        source,
    })?;
    file.write_all(contents)
        .map_err(|source| ExtractError::Io {
            path: file.path().to_path_buf(),
            source,
        })?;
    file.flush().map_err(|source| ExtractError::Io {
        path: file.path().to_path_buf(),
        source,
    })?;
    Ok(file)
}

fn parse_linkding_database(path: &Path) -> Result<ServiceCreds, ExtractError> {
    let connection = Connection::open(path).map_err(|error| ExtractError::Parse {
        service: "linkding".to_owned(),
        path: path.to_path_buf(),
        message: format!("sqlite open error: {error}"),
    })?;

    let token = connection
        .query_row(
            "SELECT key FROM authtoken_token ORDER BY created DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|error| ExtractError::Parse {
            service: "linkding".to_owned(),
            path: path.to_path_buf(),
            message: format!("failed to read authtoken_token: {error}"),
        })?;

    Ok(ServiceCreds {
        service: "linkding".to_owned(),
        url: Some("http://localhost:9090".to_owned()),
        secret: Some(token),
        env_field: "LINKDING_TOKEN".to_owned(),
        source_host: None,
        probe_host: None,
        runtime: None,
        url_verified: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_linkding_db() -> Vec<u8> {
        let path = std::env::temp_dir().join(format!(
            "lab-extract-linkding-test-{}-{}.sqlite3",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let connection = Connection::open(&path).expect("create temp sqlite db");
        connection
            .execute_batch(
                "
                CREATE TABLE authtoken_token (
                    key varchar(40) PRIMARY KEY,
                    created datetime NOT NULL,
                    user_id integer NOT NULL
                );
                INSERT INTO authtoken_token (key, created, user_id)
                VALUES
                    ('older-token', '2023-09-15 01:58:00.656565', 1),
                    ('newer-token', '2023-09-15 02:08:34.370323', 2);
                ",
            )
            .expect("seed authtoken_token");
        drop(connection);
        let bytes = fs::read(&path).expect("read sqlite bytes");
        fs::remove_file(path).expect("remove temp sqlite db");
        bytes
    }

    #[test]
    fn happy_path_extracts_newest_linkding_token() {
        let creds = LinkdingParser
            .parse(&make_linkding_db())
            .expect("parse linkding db");

        assert_eq!(creds.service, "linkding");
        assert_eq!(creds.url.as_deref(), Some("http://localhost:9090"));
        assert_eq!(creds.secret.as_deref(), Some("newer-token"));
        assert_eq!(creds.env_field, "LINKDING_TOKEN");
    }

    #[test]
    fn missing_token_table_returns_error() {
        let path = std::env::temp_dir().join(format!(
            "lab-extract-linkding-empty-{}-{}.sqlite3",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        Connection::open(&path).expect("create temp sqlite db");
        let bytes = fs::read(&path).expect("read sqlite bytes");
        fs::remove_file(path).expect("remove temp sqlite db");

        let error = LinkdingParser
            .parse(&bytes)
            .expect_err("missing token table");
        assert!(matches!(error, ExtractError::Parse { .. }));
    }
}

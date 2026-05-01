use lab_apis::neo4j::{Neo4jConfig, Neo4jError, SanitizedUri};

#[test]
fn sanitized_uri_display_never_exposes_password() {
    let uri = SanitizedUri::parse("bolt://neo4j:very-secret@localhost:7687").unwrap();

    assert!(!uri.to_string().contains("very-secret"));
    assert!(!format!("{uri:?}").contains("very-secret"));
}

#[test]
fn rejects_self_signed_tls_bypass_scheme() {
    let err = SanitizedUri::parse("bolt+ssc://localhost:7687").unwrap_err();

    assert!(matches!(err, Neo4jError::InsecureScheme(_)));
}

#[test]
fn validates_required_auth_fields() {
    let config = Neo4jConfig {
        url: "neo4j://localhost:7687".into(),
        user: "neo4j".into(),
        password: String::new(),
        database: None,
        pool_size: 16,
        ca_cert_path: None,
    };

    assert!(matches!(
        config.validate(),
        Err(Neo4jError::InvalidParam(_))
    ));
}

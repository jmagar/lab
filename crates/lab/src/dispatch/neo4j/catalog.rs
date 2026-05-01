use lab_apis::core::action::{ActionSpec, ParamSpec};

const PARAM_STATEMENT: ParamSpec = ParamSpec {
    name: "statement",
    ty: "string",
    required: true,
    description: "Cypher statement. Use parameters; do not concatenate user input into Cypher.",
};
const PARAM_PARAMETERS: ParamSpec = ParamSpec {
    name: "parameters",
    ty: "object",
    required: false,
    description: "Cypher parameter object",
};

/// Action catalog for the `neo4j` service.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "schema",
        description: "Return the parameter schema for a named action",
        destructive: false,
        returns: "Schema",
        params: &[ParamSpec {
            name: "action",
            ty: "string",
            required: true,
            description: "Action name to describe",
        }],
    },
    ActionSpec {
        name: "cypher.read",
        description: "Execute a read-only Cypher statement with parameter substitution",
        destructive: false,
        returns: "CypherResponse",
        params: &[PARAM_STATEMENT, PARAM_PARAMETERS],
    },
    ActionSpec {
        name: "cypher.write",
        description: "Execute a write Cypher statement with parameter substitution",
        destructive: true,
        returns: "WriteResponse",
        params: &[PARAM_STATEMENT, PARAM_PARAMETERS],
    },
    ActionSpec {
        name: "schema.labels",
        description: "List labels",
        destructive: false,
        returns: "CypherResponse",
        params: &[],
    },
    ActionSpec {
        name: "schema.relationships",
        description: "List relationship types",
        destructive: false,
        returns: "CypherResponse",
        params: &[],
    },
    ActionSpec {
        name: "schema.constraints",
        description: "List constraints",
        destructive: false,
        returns: "CypherResponse",
        params: &[],
    },
    ActionSpec {
        name: "schema.indexes",
        description: "List indexes",
        destructive: false,
        returns: "CypherResponse",
        params: &[],
    },
    ActionSpec {
        name: "db.list",
        description: "List databases",
        destructive: false,
        returns: "CypherResponse",
        params: &[],
    },
    ActionSpec {
        name: "db.info",
        description: "Show database info",
        destructive: false,
        returns: "CypherResponse",
        params: &[ParamSpec {
            name: "database",
            ty: "string",
            required: false,
            description: "Optional database name",
        }],
    },
    ActionSpec {
        name: "server.status",
        description: "Fetch Neo4j server component status",
        destructive: false,
        returns: "CypherResponse",
        params: &[],
    },
    ActionSpec {
        name: "txn.run",
        description: "Run multiple parameterized statements in read or write mode",
        destructive: true,
        returns: "object",
        params: &[
            ParamSpec {
                name: "statements",
                ty: "array",
                required: true,
                description: "Array of {statement, parameters}",
            },
            ParamSpec {
                name: "mode",
                ty: "string",
                required: true,
                description: "`r` for sequential read statements, `w` for one write transaction",
            },
        ],
    },
];

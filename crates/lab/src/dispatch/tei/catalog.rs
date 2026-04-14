use lab_apis::core::action::{ActionSpec, ParamSpec};

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
        name: "server.health",
        description: "Check whether the TEI server is healthy",
        destructive: false,
        returns: "void",
        params: &[],
    },
    ActionSpec {
        name: "server.info",
        description: "Get served model and runtime metadata",
        destructive: false,
        returns: "Info",
        params: &[],
    },
    ActionSpec {
        name: "embed.create",
        description: "Generate embeddings for one or more input strings",
        destructive: false,
        returns: "number[][]",
        params: &[
            ParamSpec {
                name: "input",
                ty: "string",
                required: false,
                description: "Single input string shortcut",
            },
            ParamSpec {
                name: "inputs",
                ty: "json",
                required: false,
                description: "Array of input strings",
            },
            ParamSpec {
                name: "normalize",
                ty: "bool",
                required: false,
                description: "Whether to normalize the returned vectors",
            },
            ParamSpec {
                name: "truncate",
                ty: "bool",
                required: false,
                description: "Whether to truncate overlong inputs",
            },
            ParamSpec {
                name: "payload",
                ty: "json",
                required: false,
                description: "Full request body override",
            },
        ],
    },
    ActionSpec {
        name: "embed.rerank",
        description: "Rerank texts against a query (POST /rerank). Max 100 texts per call — split larger batches across multiple requests.",
        destructive: false,
        returns: "RerankHit[]",
        params: &[
            ParamSpec {
                name: "query",
                ty: "string",
                required: true,
                description: "Query string to rank texts against",
            },
            ParamSpec {
                name: "texts",
                ty: "json",
                required: true,
                description: "Array of candidate texts (max 100)",
            },
            ParamSpec {
                name: "truncate",
                ty: "bool",
                required: false,
                description: "Whether to truncate overlong inputs",
            },
            ParamSpec {
                name: "raw_scores",
                ty: "bool",
                required: false,
                description: "Return raw logit scores instead of probabilities",
            },
        ],
    },
    ActionSpec {
        name: "embed.tokenize",
        description: "Tokenize one or more input strings (POST /tokenize). Returns token-id sequences.",
        destructive: false,
        returns: "u32[][]",
        params: &[
            ParamSpec {
                name: "inputs",
                ty: "json",
                required: true,
                description: "String or array of strings to tokenize",
            },
            ParamSpec {
                name: "add_special_tokens",
                ty: "bool",
                required: false,
                description: "Whether to add model special tokens (e.g. CLS/SEP)",
            },
        ],
    },
    ActionSpec {
        name: "embed.similarity",
        description: "Compute pairwise similarity scores for sentence pairs (POST /similarity). Inputs must be an array of [string, string] pairs.",
        destructive: false,
        returns: "f32[]",
        params: &[ParamSpec {
            name: "inputs",
            ty: "json",
            required: true,
            description: "Array of [sentence_a, sentence_b] string pairs",
        }],
    },
    ActionSpec {
        name: "embed.sparse",
        description: "Generate sparse (SPLADE-style) embeddings for one or more inputs (POST /embed_sparse).",
        destructive: false,
        returns: "SparseToken[][]",
        params: &[
            ParamSpec {
                name: "inputs",
                ty: "json",
                required: true,
                description: "String or array of strings to embed",
            },
            ParamSpec {
                name: "truncate",
                ty: "bool",
                required: false,
                description: "Whether to truncate overlong inputs",
            },
        ],
    },
    ActionSpec {
        name: "embed.openai",
        description: "Generate embeddings via the OpenAI-compatible endpoint (POST /v1/embeddings). Body and response are passed through as raw JSON.",
        destructive: false,
        returns: "json",
        params: &[ParamSpec {
            name: "body",
            ty: "json",
            required: true,
            description: "Full OpenAI-compatible request body (e.g. {model, input})",
        }],
    },
];

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
        name: "model.list",
        description: "List available models",
        destructive: false,
        returns: "Model[]",
        params: &[],
    },
    ActionSpec {
        name: "chat.complete",
        description: "Create a chat completion",
        destructive: false,
        returns: "ChatCompletionResponse",
        params: &[
            ParamSpec {
                name: "model",
                ty: "string",
                required: true,
                description: "Model identifier (e.g. gpt-4o-mini, llama3)",
            },
            ParamSpec {
                name: "messages",
                ty: "json",
                required: true,
                description: "Array of chat messages: [{\"role\":\"user\",\"content\":\"...\"}]",
            },
            ParamSpec {
                name: "temperature",
                ty: "number",
                required: false,
                description: "Sampling temperature (0.0 – 2.0)",
            },
            ParamSpec {
                name: "max_tokens",
                ty: "integer",
                required: false,
                description: "Maximum tokens to generate",
            },
        ],
    },
    ActionSpec {
        name: "embed.create",
        description: "Create embeddings for one or more input strings",
        destructive: false,
        returns: "EmbeddingsResponse",
        params: &[
            ParamSpec {
                name: "model",
                ty: "string",
                required: true,
                description: "Embedding model identifier (e.g. text-embedding-3-small)",
            },
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
                description: "Array of input strings (use input or inputs, not both)",
            },
            ParamSpec {
                name: "dimensions",
                ty: "integer",
                required: false,
                description: "Number of dimensions for the output embeddings",
            },
        ],
    },
    ActionSpec {
        name: "server.health",
        description: "Check whether the OpenAI API (or compatible server) is reachable",
        destructive: false,
        returns: "void",
        params: &[],
    },
];

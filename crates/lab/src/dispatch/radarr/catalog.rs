use lab_apis::core::action::{ActionSpec, ParamSpec};

use super::{calendar, commands, config, history, movies, queue, system};

pub fn actions() -> &'static [ActionSpec] {
    static ACTIONS: std::sync::LazyLock<&'static [ActionSpec]> = std::sync::LazyLock::new(|| {
        let mut all = vec![
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
        ];
        all.extend_from_slice(system::ACTIONS);
        all.extend_from_slice(movies::ACTIONS);
        all.extend_from_slice(queue::ACTIONS);
        all.extend_from_slice(calendar::ACTIONS);
        all.extend_from_slice(commands::ACTIONS);
        all.extend_from_slice(history::ACTIONS);
        all.extend_from_slice(config::ACTIONS);
        Vec::leak(all)
    });
    &ACTIONS
}

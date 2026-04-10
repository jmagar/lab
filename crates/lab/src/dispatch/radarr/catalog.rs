use lab_apis::core::action::ActionSpec;

use super::{calendar, commands, config, history, movies, queue, system};

pub fn actions() -> &'static [ActionSpec] {
    static ACTIONS: std::sync::LazyLock<&'static [ActionSpec]> = std::sync::LazyLock::new(|| {
        let mut all = vec![ActionSpec {
            name: "help",
            description: "Show this action catalog",
            destructive: false,
            returns: "Catalog",
            params: &[],
        }];
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

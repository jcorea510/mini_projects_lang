use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Attack,
    Move,
    Die,
    Heal,
}

/// Very small command parser: turns a string into an `Action`.
pub fn parse_action(input: &str) -> Option<Action> {
    match input.trim().to_lowercase().as_str() {
        "attack" => Some(Action::Attack),
        "move" => Some(Action::Move),
        "die" => Some(Action::Die),
        "heal" | "healt" => Some(Action::Heal),
        _ => None,
    }
}

/// Simple command registry: map an `Action` to a closure and execute it.
#[derive(Default)]
pub struct Commands {
    handlers: HashMap<Action, Box<dyn Fn()>>,
}

impl Commands {
    pub fn add_command<F>(&mut self, action: Action, func: F)
    where
        F: Fn() + 'static,
    {
        self.handlers.insert(action, Box::new(func));
    }

    pub fn execute(&self, action: Action) {
        if let Some(handler) = self.handlers.get(&action) {
            handler();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_actions() {
        assert_eq!(parse_action("attack"), Some(Action::Attack));
        assert_eq!(parse_action(" move "), Some(Action::Move));
        assert_eq!(parse_action("HEAL"), Some(Action::Heal));
        assert_eq!(parse_action("unknown"), None);
    }
}

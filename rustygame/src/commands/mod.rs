use std::collections::{HashMap, VecDeque};

use crate::ecs::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Attack,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Die,
    Heal,
}

/// Very small command parser: turns a string into an `Action`.
pub fn parse_action(input: &str) -> Option<Action> {
    match input.trim().to_lowercase().as_str() {
        "attack" => Some(Action::Attack),
        "move_left" => Some(Action::MoveLeft),
        "move_right" => Some(Action::MoveRight),
        "move_up" => Some(Action::MoveUp),
        "move_down" => Some(Action::MoveDown),
        "die" => Some(Action::Die),
        "heal" | "healt" => Some(Action::Heal),
        _ => None,
    }
}

/// Simple command registry: map an `Action` to a closure and execute it via a queue.
#[derive(Default)]
pub struct Commands {
    handlers: HashMap<Action, Box<dyn Fn(&mut World)>>,
    queue: VecDeque<Action>,
}

impl Commands {
    /// Register a handler for a given action.
    pub fn add_command<F>(&mut self, action: Action, func: F)
    where
        F: Fn(&mut World) + 'static,
    {
        self.handlers.insert(action, Box::new(func));
    }

    /// Enqueue an action to be executed later.
    pub fn enqueue(&mut self, action: Action) {
        self.queue.push_back(action);
    }

    /// Execute all queued actions in order, mutating the world.
    pub fn process_queue(&mut self, world: &mut World) {
        while let Some(action) = self.queue.pop_front() {
            if let Some(handler) = self.handlers.get(&action) {
                handler(world);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_actions() {
        assert_eq!(parse_action("attack"), Some(Action::Attack));
        assert_eq!(parse_action("move_left"), Some(Action::MoveLeft));
        assert_eq!(parse_action("move_right"), Some(Action::MoveRight));
        assert_eq!(parse_action("move_up"), Some(Action::MoveUp));
        assert_eq!(parse_action("move_down"), Some(Action::MoveDown));
        assert_eq!(parse_action("HEAL"), Some(Action::Heal));
        assert_eq!(parse_action("unknown"), None);
    }
}

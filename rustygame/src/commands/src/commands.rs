use std::default::Default;
use std::collections::HashMap;
use std::ops::Fn;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Attack,
    Move,
    Die,
    Healt,
}

#[derive(Default)]
pub struct Commands {
    command_map: HashMap<Action, Box<dyn Fn()>>,
} 

impl Commands {
    pub fn add_command<F>(&mut self, action: Action, func: &'static F) where F: Fn() {
        self.command_map.insert(action, Box::new(func));
    }

    pub fn execute(&self, action: Action) {
        self.command_map.get(&action).unwrap();
    }
}


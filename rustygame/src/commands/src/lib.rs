pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

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
    pub fn add_command<F>(&mut self, action: Action, func: F) where F: Fn() + 'static {
        self.command_map.insert(action, Box::new(func));
    }

    pub fn execute(&self, action: Action) {
        self.command_map.get(&action).unwrap()();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    
    #[test]
    fn command_adder() {
        let mut commands = Commands::default();
        let attack = || {println!("I'm attacking");};
        commands.add_command(Action::Attack, attack);

        let move_entity = || {println!("I'm moving");};
        let die = || {println!("I'm dying");};
        let healt = || {println!("I'm healing");};
        commands.add_command(Action::Move, move_entity);
        commands.add_command(Action::Die, die);
        commands.add_command(Action::Healt, healt);

        commands.execute(Action::Attack);
        commands.execute(Action::Move);
        commands.execute(Action::Die);
        commands.execute(Action::Healt);
    }
}

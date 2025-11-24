use std::fmt;

#[derive(PartialEq, Eq, Hash)]
pub enum EntityID {
    Player,
    SlimeSimple,
    SlimeCold,
    SlimeFire,
}

impl fmt::Display for EntityID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match self {
            EntityID::Player => "Player",
            EntityID::SlimeSimple => "SlimeSimple",
            EntityID::SlimeCold=> "SlimeCold",
            EntityID::SlimeFire => "SlimeFire",
        };
        write!(f, "{id}")
    }
}


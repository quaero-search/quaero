use std::fmt::Debug;

use nanoid::nanoid;

/// A unique identifier for a particular engine.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EngineId {
    name: String,
    id: String,
}

impl EngineId {
    /// Creates a identifier comprising of an engine name and a unique random string.
    pub fn from_name(name: String) -> Self {
        Self {
            name,
            id: nanoid!(10),
        }
    }
}

impl Debug for EngineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineId({}:{})", self.name, self.id)
    }
}

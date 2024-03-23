use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Vertex {
    pub user_id: String,
    pub everyone_else: HashMap<Vertex, u32>,
}

impl Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
    }
}

impl Vertex {
    pub fn new(name: &impl ToString) -> Self {
        Self {
            user_id: name.to_string(),
            ..Default::default()
        }
    }
}

impl Vertex {
    pub fn add(&mut self, other: Self) {
        let old = self.everyone_else.get(&other).unwrap_or(&0);
        self.everyone_else.insert(other, old + 1);
    }
}

use std::fmt::Display;

pub struct Conditions {
    word: String,
    pub items: Vec<String>,
}

impl Conditions {
    /// New AND condition set.
    pub fn new_and() -> Self {
        Self {
            word: "AND".to_string(),
            items: Vec::new(),
        }
    }

    /// New OR condition set.
    pub fn new_or() -> Self {
        Self {
            word: "OR".to_string(),
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, condition: String) {
        if !condition.is_empty() {
            self.items.push(condition);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Display for Conditions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.items.is_empty() {
            write!(f, "")
        } else {
            write!(
                f,
                "({})",
                self.items.join(format!(") {} (", self.word).as_str())
            )
        }
    }
}

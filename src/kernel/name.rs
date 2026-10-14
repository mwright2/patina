use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    pub fn new(name: &str) -> Option<Self> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(Name(trimmed.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_whitespace() {
        let name = Name::new("  Joint Checking \n").unwrap();
        assert_eq!(name.as_str(), "Joint Checking");
    }

    #[test]
    fn rejects_blank_names() {
        for input in ["", "   ", "\t\n"] {
            assert_eq!(Name::new(input), None, "input: {input:?}");
        }
    }
}

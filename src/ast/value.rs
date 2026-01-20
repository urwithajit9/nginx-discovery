//! Value types for NGINX directive arguments

use std::fmt;

/// Represents a value in an NGINX directive
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    /// Plain unquoted string: `nginx`
    Literal(String),

    /// Single-quoted string: `'hello world'`
    SingleQuoted(String),

    /// Double-quoted string: `"hello world"`
    DoubleQuoted(String),

    /// Variable reference: `$remote_addr`
    Variable(String),
}

impl Value {
    /// Create a new literal value
    pub fn literal(s: impl Into<String>) -> Self {
        Self::Literal(s.into())
    }

    /// Create a new single-quoted value
    pub fn single_quoted(s: impl Into<String>) -> Self {
        Self::SingleQuoted(s.into())
    }

    /// Create a new double-quoted value
    pub fn double_quoted(s: impl Into<String>) -> Self {
        Self::DoubleQuoted(s.into())
    }

    /// Create a new variable value
    pub fn variable(s: impl Into<String>) -> Self {
        Self::Variable(s.into())
    }

    /// Get the inner string value, regardless of type
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Literal(s)
            | Self::SingleQuoted(s)
            | Self::DoubleQuoted(s)
            | Self::Variable(s) => s,
        }
    }

    /// Check if this is a variable
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    /// Check if this is quoted (single or double)
    #[must_use]
    pub fn is_quoted(&self) -> bool {
        matches!(self, Self::SingleQuoted(_) | Self::DoubleQuoted(_))
    }

    /// Get the value as it would appear in the config file
    #[must_use]
    pub fn to_config_string(&self) -> String {
        match self {
            Self::Literal(s) => s.clone(),
            Self::SingleQuoted(s) => format!("'{s}'"),
            Self::DoubleQuoted(s) => format!("\"{s}\""),
            Self::Variable(s) => format!("${s}"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::Literal(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::Literal(s.to_string())
    }
}

impl From<&String> for Value {
    fn from(s: &String) -> Self {
        Self::Literal(s.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_value() {
        let val = Value::literal("nginx");
        assert_eq!(val.as_str(), "nginx");
        assert!(!val.is_variable());
        assert!(!val.is_quoted());
        assert_eq!(val.to_config_string(), "nginx");
    }

    #[test]
    fn test_single_quoted_value() {
        let val = Value::single_quoted("hello world");
        assert_eq!(val.as_str(), "hello world");
        assert!(val.is_quoted());
        assert_eq!(val.to_config_string(), "'hello world'");
    }

    #[test]
    fn test_double_quoted_value() {
        let val = Value::double_quoted("hello world");
        assert_eq!(val.as_str(), "hello world");
        assert!(val.is_quoted());
        assert_eq!(val.to_config_string(), "\"hello world\"");
    }

    #[test]
    fn test_variable_value() {
        let val = Value::variable("remote_addr");
        assert_eq!(val.as_str(), "remote_addr");
        assert!(val.is_variable());
        assert!(!val.is_quoted());
        assert_eq!(val.to_config_string(), "$remote_addr");
    }

    #[test]
    fn test_value_display() {
        let val = Value::literal("test");
        assert_eq!(val.to_string(), "test");

        let var = Value::variable("host");
        assert_eq!(var.to_string(), "host");
    }

    #[test]
    fn test_value_from_string() {
        let val: Value = "nginx".into();
        assert_eq!(val.as_str(), "nginx");

        let val2: Value = String::from("test").into();
        assert_eq!(val2.as_str(), "test");
    }

    #[test]
    fn test_value_equality() {
        let val1 = Value::literal("test");
        let val2 = Value::literal("test");
        assert_eq!(val1, val2);

        let val3 = Value::single_quoted("test");
        assert_ne!(val1, val3); // Different types
    }
}

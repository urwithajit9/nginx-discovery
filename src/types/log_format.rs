//! Log format type

/// Represents an NGINX `log_format` directive
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogFormat {
    /// Format name
    pub name: String,

    /// Format pattern string
    pub pattern: String,

    /// Extracted variable names from the pattern
    pub variables: Vec<String>,
}

impl LogFormat {
    /// Create a new log format
    #[must_use]
    pub fn new(name: impl Into<String>, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let variables = extract_variables(&pattern);

        Self {
            name: name.into(),
            pattern,
            variables,
        }
    }

    /// Get the format name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the format pattern
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Get variable names used in this format
    #[must_use]
    pub fn variables(&self) -> &[String] {
        &self.variables
    }
}

/// Extract variable names from a log format pattern
fn extract_variables(pattern: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut var_name = String::new();

            // Handle ${var_name} or $var_name
            if chars.peek() == Some(&'{') {
                chars.next(); // Skip {
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next(); // Skip }
                        break;
                    }
                    var_name.push(c);
                    chars.next();
                }
            } else {
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        var_name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            if !var_name.is_empty() {
                variables.push(var_name);
            }
        }
    }

    variables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variables() {
        let pattern = "$remote_addr - $remote_user [$time_local]";
        let vars = extract_variables(pattern);

        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0], "remote_addr");
        assert_eq!(vars[1], "remote_user");
        assert_eq!(vars[2], "time_local");
    }

    #[test]
    fn test_extract_variables_with_braces() {
        let pattern = "${host} - ${request_uri}";
        let vars = extract_variables(pattern);

        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0], "host");
        assert_eq!(vars[1], "request_uri");
    }

    #[test]
    fn test_log_format_creation() {
        let format = LogFormat::new("combined", "$remote_addr $request");

        assert_eq!(format.name(), "combined");
        assert_eq!(format.variables().len(), 2);
    }
}

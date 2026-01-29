// src/export/format.rs
//! Export format definitions and trait

use std::fmt;

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExportFormat {
    /// JSON format (compact or pretty)
    Json,

    /// YAML format
    Yaml,

    /// TOML format (requires `export-toml` feature)
    #[cfg(feature = "export-toml")]
    Toml,

    /// Markdown documentation format (requires `export-markdown` feature)
    #[cfg(feature = "export-markdown")]
    Markdown,
}

impl ExportFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            #[cfg(feature = "export-toml")]
            Self::Toml => "toml",
            #[cfg(feature = "export-markdown")]
            Self::Markdown => "md",
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Yaml => "application/x-yaml",
            #[cfg(feature = "export-toml")]
            Self::Toml => "application/toml",
            #[cfg(feature = "export-markdown")]
            Self::Markdown => "text/markdown",
        }
    }

    /// Check if this format supports pretty printing
    pub fn supports_pretty(&self) -> bool {
        match self {
            Self::Json => true,
            Self::Yaml => false, // YAML is always pretty
            #[cfg(feature = "export-toml")]
            Self::Toml => true,
            #[cfg(feature = "export-markdown")]
            Self::Markdown => false, // Markdown is always formatted
        }
    }

    /// All available formats (depends on enabled features)
    pub fn all() -> Vec<Self> {
        vec![
            Self::Json,
            Self::Yaml,
            #[cfg(feature = "export-toml")]
            Self::Toml,
            #[cfg(feature = "export-markdown")]
            Self::Markdown,
        ]
    }
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Json
    }
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            #[cfg(feature = "export-toml")]
            Self::Toml => write!(f, "toml"),
            #[cfg(feature = "export-markdown")]
            Self::Markdown => write!(f, "markdown"),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            #[cfg(feature = "export-toml")]
            "toml" => Ok(Self::Toml),
            #[cfg(feature = "export-markdown")]
            "markdown" | "md" => Ok(Self::Markdown),
            _ => Err(format!("Unknown format: {}. Available: {}", s,
                Self::all().iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

/// Trait for custom exporters
pub trait Exporter {
    /// Export configuration to the writer
    fn export(&self, config: &crate::Config, writer: &mut dyn std::io::Write) -> crate::Result<()>;

    /// Get format name
    fn format_name(&self) -> &str;

    /// Get file extension
    fn extension(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_display() {
        assert_eq!(ExportFormat::Json.to_string(), "json");
        assert_eq!(ExportFormat::Yaml.to_string(), "yaml");
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!("json".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!("yaml".parse::<ExportFormat>().unwrap(), ExportFormat::Yaml);
        assert_eq!("yml".parse::<ExportFormat>().unwrap(), ExportFormat::Yaml);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Yaml.extension(), "yaml");
    }

    #[test]
    fn test_format_mime_type() {
        assert_eq!(ExportFormat::Json.mime_type(), "application/json");
        assert_eq!(ExportFormat::Yaml.mime_type(), "application/x-yaml");
    }
}
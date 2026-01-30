// src/export/format.rs
//! Export format definitions and trait.
//!
//! This module defines the supported export formats for NGINX configurations
//! and provides utilities for working with different output formats.

use std::fmt;

/// Supported export formats for NGINX configurations.
///
/// Different formats are available depending on which features are enabled:
/// - `json` and `yaml` are always available when the `serde` feature is enabled
/// - `toml` requires the `export-toml` feature
/// - `markdown` requires the `export-markdown` feature
///
/// # Examples
///
/// ```
/// use nginx_discovery::export::ExportFormat;
///
/// let format = ExportFormat::Json;
/// assert_eq!(format.extension(), "json");
/// assert_eq!(format.mime_type(), "application/json");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExportFormat {
    /// JSON format (compact or pretty).
    ///
    /// Supports both compact single-line output and pretty-printed formatting.
    #[default]
    Json,

    /// YAML format.
    ///
    /// Always outputs in human-readable format.
    Yaml,

    /// TOML format (requires `export-toml` feature).
    ///
    /// Supports both compact and pretty-printed formatting.
    #[cfg(feature = "export-toml")]
    Toml,

    /// Markdown documentation format (requires `export-markdown` feature).
    ///
    /// Generates human-readable documentation with sections and formatting.
    #[cfg(feature = "export-markdown")]
    Markdown,
}

impl ExportFormat {
    /// Returns the file extension for this format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportFormat;
    ///
    /// assert_eq!(ExportFormat::Json.extension(), "json");
    /// assert_eq!(ExportFormat::Yaml.extension(), "yaml");
    /// ```
    #[must_use]
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

    /// Returns the MIME type for this format.
    ///
    /// This is useful for setting HTTP `Content-Type` headers when serving
    /// exported configurations over HTTP.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportFormat;
    ///
    /// assert_eq!(ExportFormat::Json.mime_type(), "application/json");
    /// assert_eq!(ExportFormat::Yaml.mime_type(), "application/x-yaml");
    /// ```
    #[must_use]
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

    /// Returns whether this format supports pretty printing.
    ///
    /// Some formats like YAML are always formatted in a human-readable way,
    /// while others like JSON can be either compact or pretty-printed.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportFormat;
    ///
    /// assert!(ExportFormat::Json.supports_pretty());
    /// assert!(!ExportFormat::Yaml.supports_pretty()); // Always pretty
    /// ```
    #[must_use]
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

    /// Returns all available export formats.
    ///
    /// The returned formats depend on which features are enabled at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportFormat;
    ///
    /// let formats = ExportFormat::all();
    /// assert!(formats.contains(&ExportFormat::Json));
    /// assert!(formats.contains(&ExportFormat::Yaml));
    /// ```
    #[must_use]
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

    /// Parses an export format from a string.
    ///
    /// # Supported Values
    ///
    /// - `"json"` → `ExportFormat::Json`
    /// - `"yaml"` or `"yml"` → `ExportFormat::Yaml`
    /// - `"toml"` → `ExportFormat::Toml` (if `export-toml` feature enabled)
    /// - `"markdown"` or `"md"` → `ExportFormat::Markdown` (if `export-markdown` feature enabled)
    ///
    /// Matching is case-insensitive.
    ///
    /// # Errors
    ///
    /// Returns an error message listing available formats if the input string
    /// doesn't match any known format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportFormat;
    ///
    /// let format: ExportFormat = "json".parse().unwrap();
    /// assert_eq!(format, ExportFormat::Json);
    ///
    /// let format: ExportFormat = "YAML".parse().unwrap();
    /// assert_eq!(format, ExportFormat::Yaml);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            #[cfg(feature = "export-toml")]
            "toml" => Ok(Self::Toml),
            #[cfg(feature = "export-markdown")]
            "markdown" | "md" => Ok(Self::Markdown),
            _ => Err(format!(
                "Unknown format: {s}. Available: {}",
                Self::all()
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

/// Trait for implementing custom exporters.
///
/// This trait allows you to create custom export formats beyond the built-in ones.
/// Implement this trait to add support for new output formats.
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::export::Exporter;
/// use nginx_discovery::ast::Config;
/// use std::io::Write;
///
/// struct CustomExporter;
///
/// impl Exporter for CustomExporter {
///     fn export(&self, config: &Config, writer: &mut dyn Write) -> nginx_discovery::Result<()> {
///         writeln!(writer, "Custom format output")?;
///         Ok(())
///     }
///
///     fn format_name(&self) -> &str {
///         "custom"
///     }
///
///     fn extension(&self) -> &str {
///         "custom"
///     }
/// }
/// ```
pub trait Exporter {
    /// Exports the configuration to the provided writer.
    ///
    /// # Arguments
    ///
    /// * `config` - The NGINX configuration to export
    /// * `writer` - The destination to write the exported data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Writing to the output fails
    /// - The configuration cannot be serialized in this format
    /// - Any I/O errors occur during export
    fn export(
        &self,
        config: &crate::ast::Config,
        writer: &mut dyn std::io::Write,
    ) -> crate::Result<()>;

    /// Returns the human-readable name of this format.
    ///
    /// # Examples
    ///
    /// ```text
    /// "json", "yaml", "custom"
    /// ```
    fn format_name(&self) -> &str;

    /// Returns the file extension for this format (without the dot).
    ///
    /// # Examples
    ///
    /// ```text
    /// "json", "yaml", "txt"
    /// ```
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
    fn test_format_from_str_case_insensitive() {
        assert_eq!("JSON".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!("YaML".parse::<ExportFormat>().unwrap(), ExportFormat::Yaml);
    }

    #[test]
    fn test_format_from_str_invalid() {
        let result = "invalid".parse::<ExportFormat>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown format"));
        assert!(err.contains("Available"));
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

    #[test]
    fn test_format_supports_pretty() {
        assert!(ExportFormat::Json.supports_pretty());
        assert!(!ExportFormat::Yaml.supports_pretty());
    }

    #[test]
    fn test_format_all() {
        let formats = ExportFormat::all();
        assert!(formats.contains(&ExportFormat::Json));
        assert!(formats.contains(&ExportFormat::Yaml));
        assert!(!formats.is_empty());
    }

    #[test]
    fn test_format_default() {
        assert_eq!(ExportFormat::default(), ExportFormat::Json);
    }

    #[cfg(feature = "export-toml")]
    #[test]
    fn test_toml_format() {
        assert_eq!(ExportFormat::Toml.extension(), "toml");
        assert_eq!(ExportFormat::Toml.mime_type(), "application/toml");
        assert!(ExportFormat::Toml.supports_pretty());
    }

    #[cfg(feature = "export-markdown")]
    #[test]
    fn test_markdown_format() {
        assert_eq!(ExportFormat::Markdown.extension(), "md");
        assert_eq!(ExportFormat::Markdown.mime_type(), "text/markdown");
        assert!(!ExportFormat::Markdown.supports_pretty());
    }
}

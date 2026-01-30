// src/export/options.rs
//! Export options and builder pattern.
//!
//! This module provides configuration options for exporting NGINX configurations
//! in various formats. The builder pattern allows for flexible and readable
//! option construction.
//!
//! # Examples
//!
//! ```
//! use nginx_discovery::export::{ExportOptions, ExportFormat};
//!
//! // Using default options
//! let options = ExportOptions::default();
//!
//! // Using the builder
//! let options = ExportOptions::builder()
//!     .format(ExportFormat::Json)
//!     .pretty(true)
//!     .include_metadata(true)
//!     .build();
//! ```

use super::{ExportFormat, Filter};

/// Export options for controlling output format and content.
///
/// This struct controls how NGINX configurations are exported,
/// including format selection, formatting options, and content filtering.
///
/// # Examples
///
/// ```
/// use nginx_discovery::export::{ExportOptions, ExportFormat};
///
/// let options = ExportOptions {
///     format: ExportFormat::Json,
///     pretty: true,
///     include_metadata: true,
///     include_comments: false,
///     compact: false,
///     filter: None,
///     template: None,
/// };
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ExportOptions {
    /// Output format (JSON, YAML, TOML, Markdown)
    pub format: ExportFormat,

    /// Enable pretty printing (if supported by format)
    pub pretty: bool,

    /// Include metadata like timestamps and counts
    pub include_metadata: bool,

    /// Include comments from original configuration
    pub include_comments: bool,

    /// Minimize whitespace in output
    pub compact: bool,

    /// Optional filter to apply before export
    pub filter: Option<Filter>,

    /// Custom template for markdown/html exports
    pub template: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::default(),
            pretty: true,
            include_metadata: true,
            include_comments: false,
            compact: false,
            filter: None,
            template: None,
        }
    }
}

impl ExportOptions {
    /// Creates a new builder for constructing export options.
    ///
    /// The builder pattern provides a clean, readable way to configure
    /// export options with only the settings you need.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::{ExportOptions, ExportFormat};
    ///
    /// let options = ExportOptions::builder()
    ///     .format(ExportFormat::Yaml)
    ///     .pretty(false)
    ///     .build();
    /// ```
    #[must_use]
    pub fn builder() -> ExportOptionsBuilder {
        ExportOptionsBuilder::default()
    }
}

/// Builder for `ExportOptions`.
///
/// Provides a fluent interface for constructing export options.
/// All methods return `self` to allow method chaining.
///
/// # Examples
///
/// ```
/// use nginx_discovery::export::{ExportOptions, ExportFormat, Filter, FilterType};
///
/// let filter = Filter::new(FilterType::Port, "443");
/// let options = ExportOptions::builder()
///     .format(ExportFormat::Json)
///     .pretty(true)
///     .include_metadata(false)
///     .filter(filter)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ExportOptionsBuilder {
    format: Option<ExportFormat>,
    pretty: Option<bool>,
    include_metadata: Option<bool>,
    include_comments: Option<bool>,
    compact: Option<bool>,
    filter: Option<Filter>,
    template: Option<String>,
}

impl ExportOptionsBuilder {
    /// Sets the export format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::{ExportOptions, ExportFormat};
    ///
    /// let options = ExportOptions::builder()
    ///     .format(ExportFormat::Yaml)
    ///     .build();
    /// ```
    #[must_use]
    pub fn format(mut self, format: ExportFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Enables or disables pretty printing.
    ///
    /// Pretty printing adds whitespace and formatting to make output
    /// more readable. Not all formats support this option.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportOptions;
    ///
    /// let options = ExportOptions::builder()
    ///     .pretty(false)  // Compact output
    ///     .build();
    /// ```
    #[must_use]
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = Some(pretty);
        self
    }

    /// Enables or disables metadata inclusion.
    ///
    /// Metadata includes information like generation timestamp,
    /// directive counts, and other statistics.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportOptions;
    ///
    /// let options = ExportOptions::builder()
    ///     .include_metadata(false)  // No metadata
    ///     .build();
    /// ```
    #[must_use]
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = Some(include);
        self
    }

    /// Enables or disables comment inclusion.
    ///
    /// When enabled, comments from the original configuration
    /// will be preserved in the export (if the format supports it).
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportOptions;
    ///
    /// let options = ExportOptions::builder()
    ///     .include_comments(true)
    ///     .build();
    /// ```
    #[must_use]
    pub fn include_comments(mut self, include: bool) -> Self {
        self.include_comments = Some(include);
        self
    }

    /// Enables or disables compact output.
    ///
    /// Compact output minimizes whitespace to reduce file size.
    /// This is the opposite of pretty printing.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportOptions;
    ///
    /// let options = ExportOptions::builder()
    ///     .compact(true)  // Minimize whitespace
    ///     .build();
    /// ```
    #[must_use]
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = Some(compact);
        self
    }

    /// Sets a filter to apply before export.
    ///
    /// Filters allow you to export only parts of the configuration
    /// that match specific criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::{ExportOptions, Filter, FilterType};
    ///
    /// let filter = Filter::new(FilterType::ServerName, "example.com");
    /// let options = ExportOptions::builder()
    ///     .filter(filter)
    ///     .build();
    /// ```
    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Sets a custom template for the export.
    ///
    /// Templates are used for Markdown and HTML exports to customize
    /// the output format. The template syntax depends on the format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::ExportOptions;
    ///
    /// let options = ExportOptions::builder()
    ///     .template("custom-template.md")
    ///     .build();
    /// ```
    #[must_use]
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Builds the final `ExportOptions`.
    ///
    /// Any unset options will use their default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::{ExportOptions, ExportFormat};
    ///
    /// let options = ExportOptions::builder()
    ///     .format(ExportFormat::Json)
    ///     .build();
    /// ```
    #[must_use]
    pub fn build(self) -> ExportOptions {
        let defaults = ExportOptions::default();

        ExportOptions {
            format: self.format.unwrap_or(defaults.format),
            pretty: self.pretty.unwrap_or(defaults.pretty),
            include_metadata: self.include_metadata.unwrap_or(defaults.include_metadata),
            include_comments: self.include_comments.unwrap_or(defaults.include_comments),
            compact: self.compact.unwrap_or(defaults.compact),
            filter: self.filter.or(defaults.filter),
            template: self.template.or(defaults.template),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = ExportOptions::default();
        assert_eq!(options.format, ExportFormat::Json);
        assert!(options.pretty);
        assert!(options.include_metadata);
        assert!(!options.include_comments);
        assert!(!options.compact);
    }

    #[test]
    fn test_builder() {
        let options = ExportOptions::builder()
            .format(ExportFormat::Yaml)
            .pretty(false)
            .compact(true)
            .build();

        assert_eq!(options.format, ExportFormat::Yaml);
        assert!(!options.pretty);
        assert!(options.compact);
    }

    #[test]
    fn test_builder_partial() {
        let options = ExportOptions::builder().format(ExportFormat::Json).build();

        // Should use defaults for unset fields
        assert!(options.pretty);
        assert!(options.include_metadata);
    }

    #[test]
    fn test_builder_all_options() {
        let options = ExportOptions::builder()
            .format(ExportFormat::Yaml)
            .pretty(false)
            .include_metadata(false)
            .include_comments(true)
            .compact(true)
            .template("custom.md")
            .build();

        assert_eq!(options.format, ExportFormat::Yaml);
        assert!(!options.pretty);
        assert!(!options.include_metadata);
        assert!(options.include_comments);
        assert!(options.compact);
        assert_eq!(options.template.as_deref(), Some("custom.md"));
    }

    #[test]
    fn test_builder_chaining() {
        let options = ExportOptions::builder()
            .format(ExportFormat::Json)
            .pretty(true)
            .include_metadata(true)
            .include_comments(false)
            .compact(false)
            .build();

        assert_eq!(options.format, ExportFormat::Json);
        assert!(options.pretty);
    }
}

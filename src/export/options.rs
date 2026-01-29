// src/export/options.rs
//! Export options and builder pattern

use super::{ExportFormat, Filter};

/// Export options for controlling output format and content
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Output format
    pub format: ExportFormat,

    /// Pretty print (if supported by format)
    pub pretty: bool,

    /// Include metadata (timestamps, counts, etc.)
    pub include_metadata: bool,

    /// Include comments from original config
    pub include_comments: bool,

    /// Compact output (minimize whitespace)
    pub compact: bool,

    /// Filter to apply before export
    pub filter: Option<Filter>,

    /// Custom template (for markdown/html exports)
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
    /// Create a new builder
    pub fn builder() -> ExportOptionsBuilder {
        ExportOptionsBuilder::default()
    }
}

/// Builder for ExportOptions
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
    /// Set export format
    #[must_use]
    pub fn format(mut self, format: ExportFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Enable/disable pretty printing
    #[must_use]
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = Some(pretty);
        self
    }

    /// Enable/disable metadata inclusion
    #[must_use]
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = Some(include);
        self
    }

    /// Enable/disable comment inclusion
    #[must_use]
    pub fn include_comments(mut self, include: bool) -> Self {
        self.include_comments = Some(include);
        self
    }

    /// Enable/disable compact output
    #[must_use]
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = Some(compact);
        self
    }

    /// Set filter
    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set custom template
    #[must_use]
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    /// Build the options
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
        let options = ExportOptions::builder()
            .format(ExportFormat::Json)
            .build();

        // Should use defaults for unset fields
        assert!(options.pretty);
        assert!(options.include_metadata);
    }
}
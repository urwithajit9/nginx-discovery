//! Directive AST nodes

use super::{Span, Spanned, Value};

/// A directive in the NGINX configuration
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Directive {
    /// The directive content (simple or block)
    pub item: DirectiveItem,
    /// Source location
    pub span: Span,
}

/// Directive content - either simple or block
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DirectiveItem {
    /// Simple directive: `name arg1 arg2 ...;`
    Simple {
        /// Directive name
        name: String,
        /// Arguments
        args: Vec<Value>,
    },
    /// Block directive: `name arg1 arg2 { ... }`
    Block {
        /// Directive name
        name: String,
        /// Arguments before the block
        args: Vec<Value>,
        /// Child directives
        children: Vec<Directive>,
    },
}

impl Directive {
    /// Create a new simple directive
    pub fn simple(name: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            item: DirectiveItem::Simple {
                name: name.into(),
                args: args.into_iter().map(Value::from).collect(),
            },
            span: Span::default(),
        }
    }

    /// Create a new simple directive with span
    pub fn simple_with_span(name: impl Into<String>, args: Vec<String>, span: Span) -> Self {
        Self {
            item: DirectiveItem::Simple {
                name: name.into(),
                args: args.into_iter().map(Value::from).collect(),
            },
            span,
        }
    }

    /// Create a new simple directive with values
    pub fn simple_with_values(name: impl Into<String>, args: Vec<Value>) -> Self {
        Self {
            item: DirectiveItem::Simple {
                name: name.into(),
                args,
            },
            span: Span::default(),
        }
    }

    /// Create a new block directive
    pub fn block(name: impl Into<String>, args: Vec<String>, children: Vec<Directive>) -> Self {
        Self {
            item: DirectiveItem::Block {
                name: name.into(),
                args: args.into_iter().map(Value::from).collect(),
                children,
            },
            span: Span::default(),
        }
    }

    /// Create a new block directive with span
    pub fn block_with_span(
        name: impl Into<String>,
        args: Vec<String>,
        children: Vec<Directive>,
        span: Span,
    ) -> Self {
        Self {
            item: DirectiveItem::Block {
                name: name.into(),
                args: args.into_iter().map(Value::from).collect(),
                children,
            },
            span,
        }
    }

    /// Create a new block directive with values
    pub fn block_with_values(
        name: impl Into<String>,
        args: Vec<Value>,
        children: Vec<Directive>,
    ) -> Self {
        Self {
            item: DirectiveItem::Block {
                name: name.into(),
                args,
                children,
            },
            span: Span::default(),
        }
    }

    /// Get the directive name
    #[must_use]
    pub fn name(&self) -> &str {
        match &self.item {
            DirectiveItem::Simple { name, .. } | DirectiveItem::Block { name, .. } => name,
        }
    }

    /// Get the directive arguments
    #[must_use]
    pub fn args(&self) -> &[Value] {
        match &self.item {
            DirectiveItem::Simple { args, .. } | DirectiveItem::Block { args, .. } => args,
        }
    }

    /// Get children if this is a block directive
    #[must_use]
    pub fn children(&self) -> Option<&[Directive]> {
        match &self.item {
            DirectiveItem::Block { children, .. } => Some(children),
            DirectiveItem::Simple { .. } => None,
        }
    }

    /// Get mutable children if this is a block directive
    pub fn children_mut(&mut self) -> Option<&mut Vec<Directive>> {
        match &mut self.item {
            DirectiveItem::Block { children, .. } => Some(children),
            DirectiveItem::Simple { .. } => None,
        }
    }

    /// Check if this is a block directive
    #[must_use]
    pub fn is_block(&self) -> bool {
        matches!(self.item, DirectiveItem::Block { .. })
    }

    /// Check if this is a simple directive
    #[must_use]
    pub fn is_simple(&self) -> bool {
        matches!(self.item, DirectiveItem::Simple { .. })
    }

    /// Get the first argument as a string, if it exists
    #[must_use]
    pub fn first_arg(&self) -> Option<String> {
        self.args().first().map(std::string::ToString::to_string)
    }

    /// Get all arguments as strings
    #[must_use]
    pub fn args_as_strings(&self) -> Vec<String> {
        self.args()
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Find child directives with a specific name
    #[must_use]
    pub fn find_children(&self, name: &str) -> Vec<&Directive> {
        match &self.item {
            DirectiveItem::Block { children, .. } => {
                children.iter().filter(|d| d.name() == name).collect()
            }
            DirectiveItem::Simple { .. } => Vec::new(),
        }
    }

    /// Find child directives with a specific name (mutable)
    pub fn find_children_mut(&mut self, name: &str) -> Vec<&mut Directive> {
        match &mut self.item {
            DirectiveItem::Block { children, .. } => {
                children.iter_mut().filter(|d| d.name() == name).collect()
            }
            DirectiveItem::Simple { .. } => Vec::new(),
        }
    }

    /// Recursively find all directives with a given name
    #[must_use]
    pub fn find_recursive(&self, name: &str) -> Vec<&Directive> {
        let mut result = Vec::new();
        self.find_recursive_impl(name, &mut result);
        result
    }

    fn find_recursive_impl<'a>(&'a self, name: &str, result: &mut Vec<&'a Directive>) {
        if self.name() == name {
            result.push(self);
        }
        if let Some(children) = self.children() {
            for child in children {
                child.find_recursive_impl(name, result);
            }
        }
    }
}

impl Spanned for Directive {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_directive() {
        let directive = Directive::simple("user", vec!["nginx".to_string()]);
        assert_eq!(directive.name(), "user");
        assert_eq!(directive.args().len(), 1);
        assert!(directive.is_simple());
        assert!(!directive.is_block());
        assert_eq!(directive.first_arg(), Some("nginx".to_string()));
    }

    #[test]
    fn test_simple_directive_with_span() {
        let span = Span::new(0, 10, 1, 1);
        let directive = Directive::simple_with_span("listen", vec!["80".to_string()], span);
        assert_eq!(directive.span, span);
        assert_eq!(directive.name(), "listen");
    }

    #[test]
    fn test_block_directive() {
        let directive = Directive::block("server", vec![], vec![]);
        assert_eq!(directive.name(), "server");
        assert!(directive.is_block());
        assert!(!directive.is_simple());
        assert_eq!(directive.children().unwrap().len(), 0);
    }

    #[test]
    fn test_block_with_children() {
        let children = vec![
            Directive::simple("listen", vec!["80".to_string()]),
            Directive::simple("server_name", vec!["example.com".to_string()]),
        ];
        let server = Directive::block("server", vec![], children);

        assert_eq!(server.children().unwrap().len(), 2);
        assert_eq!(server.children().unwrap()[0].name(), "listen");
    }

    #[test]
    fn test_find_children() {
        let children = vec![
            Directive::simple("listen", vec!["80".to_string()]),
            Directive::simple("server_name", vec!["example.com".to_string()]),
            Directive::simple("listen", vec!["443".to_string()]),
        ];
        let server = Directive::block("server", vec![], children);

        let listen_dirs = server.find_children("listen");
        assert_eq!(listen_dirs.len(), 2);
        assert_eq!(listen_dirs[0].first_arg(), Some("80".to_string()));
        assert_eq!(listen_dirs[1].first_arg(), Some("443".to_string()));
    }

    #[test]
    fn test_args_as_strings() {
        let directive = Directive::simple(
            "server_name",
            vec!["example.com".to_string(), "www.example.com".to_string()],
        );
        let args = directive.args_as_strings();
        assert_eq!(args, vec!["example.com", "www.example.com"]);
    }

    #[test]
    fn test_find_recursive() {
        // Build nested structure:
        // http {
        //   server {
        //     location / {
        //       access_log /var/log/1.log;
        //     }
        //   }
        //   access_log /var/log/2.log;
        // }
        let location = Directive::block(
            "location",
            vec!["/".to_string()],
            vec![Directive::simple(
                "access_log",
                vec!["/var/log/1.log".to_string()],
            )],
        );
        let server = Directive::block("server", vec![], vec![location]);
        let http = Directive::block(
            "http",
            vec![],
            vec![
                server,
                Directive::simple("access_log", vec!["/var/log/2.log".to_string()]),
            ],
        );

        let access_logs = http.find_recursive("access_log");
        assert_eq!(access_logs.len(), 2);
    }

    #[test]
    fn test_simple_directive_with_values() {
        let args = vec![Value::literal("combined"), Value::variable("remote_addr")];
        let directive = Directive::simple_with_values("log_format", args);

        assert_eq!(directive.args().len(), 2);
        assert!(directive.args()[1].is_variable());
    }
}

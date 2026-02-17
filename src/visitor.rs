use crate::ast::{Config, Directive, DirectiveItem};

/// Trait for traversing the NGINX AST.
pub trait Visitor {
    /// Entry point for visiting the entire configuration
    fn visit_config(&mut self, config: &Config) {
        for directive in &config.directives {
            self.visit_directive(directive);
        }
    }

    /// Called for every directive in the tree
    fn visit_directive(&mut self, directive: &Directive) {
        // Provide a default "walking" behavior to go deeper into blocks
        walk_directive(self, directive);
    }
}

/// Helper function to traverse into child directives (Recursive)
pub fn walk_directive<V: Visitor + ?Sized>(visitor: &mut V, directive: &Directive) {
    if let DirectiveItem::Block { children, .. } = &directive.item {
        for child in children {
            visitor.visit_directive(child);
        }
    }
}

// Example: A concrete visitor that counts how many times 'listen' is used
pub struct ListenCounter {
    pub count: usize,
}

impl Visitor for ListenCounter {
    fn visit_directive(&mut self, directive: &Directive) {
        if directive.name() == "listen" {
            self.count += 1;
        }
        // Continue walking through children if it's a block
        walk_directive(self, directive);
    }
}
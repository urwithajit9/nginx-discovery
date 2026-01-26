// src/types/location.rs
use crate::types::AccessLog;
use std::path::PathBuf;
/// Represents an NGINX location block
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Location {
    /// Location path/pattern
    pub path: String,

    /// Location modifier
    pub modifier: LocationModifier,

    /// Root directory (if specified)
    pub root: Option<PathBuf>,

    /// Proxy pass upstream (if specified)
    pub proxy_pass: Option<String>,

    /// Access logs for this location
    pub access_logs: Vec<AccessLog>,
}

impl Location {
    /// Create a new location
    pub fn new(path: impl Into<String>, modifier: LocationModifier) -> Self {
        Self {
            path: path.into(),
            modifier,
            root: None,
            proxy_pass: None,
            access_logs: Vec::new(),
        }
    }

    /// Check if this is a proxy location
    #[must_use]
    pub fn is_proxy(&self) -> bool {
        self.proxy_pass.is_some()
    }

    /// Check if this serves static files
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.root.is_some() && self.proxy_pass.is_none()
    }
}

/// Location modifier types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocationModifier {
    /// No modifier: `location /path`
    None,

    /// Exact match: `location = /path`
    Exact,

    /// Prefix match (priority): `location ^~ /path`
    PrefixPriority,

    /// Case-sensitive regex: `location ~ pattern`
    Regex,

    /// Case-insensitive regex: `location ~* pattern`
    RegexCaseInsensitive,
}

impl LocationModifier {
    /// Parse from location directive arguments
    #[must_use]
    pub fn from_args(args: &[String]) -> (Self, String) {
        if args.is_empty() {
            return (Self::None, "/".to_string());
        }

        // Check first argument for modifier
        match args[0].as_str() {
            "=" => {
                let path = args.get(1).map_or("/", String::as_str);
                (Self::Exact, path.to_string())
            }
            "^~" => {
                let path = args.get(1).map_or("/", String::as_str);
                (Self::PrefixPriority, path.to_string())
            }
            "~" => {
                let pattern = args.get(1).map_or("", String::as_str);
                (Self::Regex, pattern.to_string())
            }
            "~*" => {
                let pattern = args.get(1).map_or("", String::as_str);
                (Self::RegexCaseInsensitive, pattern.to_string())
            }
            _ => {
                // No modifier, first arg is the path
                (Self::None, args[0].clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_new() {
        let location = Location::new("/api", LocationModifier::None);
        assert_eq!(location.path, "/api");
        assert_eq!(location.modifier, LocationModifier::None);
        assert!(location.root.is_none());
        assert!(location.proxy_pass.is_none());
        assert!(location.access_logs.is_empty());
    }

    #[test]
    fn test_is_proxy_true() {
        let mut location = Location::new("/api", LocationModifier::None);
        location.proxy_pass = Some("http://backend:8080".to_string());

        assert!(location.is_proxy());
    }

    #[test]
    fn test_is_proxy_false() {
        let location = Location::new("/api", LocationModifier::None);
        assert!(!location.is_proxy());
    }

    #[test]
    fn test_is_static_true() {
        let mut location = Location::new("/static", LocationModifier::None);
        location.root = Some(PathBuf::from("/var/www/static"));

        assert!(location.is_static());
    }

    #[test]
    fn test_is_static_false_no_root() {
        let location = Location::new("/", LocationModifier::None);
        assert!(!location.is_static());
    }

    #[test]
    fn test_is_static_false_has_proxy() {
        let mut location = Location::new("/api", LocationModifier::None);
        location.root = Some(PathBuf::from("/var/www"));
        location.proxy_pass = Some("http://backend".to_string());

        assert!(!location.is_static());
    }

    #[test]
    fn test_location_modifier_none() {
        let args = vec!["/path".to_string()];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::None);
        assert_eq!(path, "/path");
    }

    #[test]
    fn test_location_modifier_exact() {
        let args = vec!["=".to_string(), "/exact".to_string()];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::Exact);
        assert_eq!(path, "/exact");
    }

    #[test]
    fn test_location_modifier_prefix_priority() {
        let args = vec!["^~".to_string(), "/static/".to_string()];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::PrefixPriority);
        assert_eq!(path, "/static/");
    }

    #[test]
    fn test_location_modifier_regex() {
        let args = vec!["~".to_string(), r"\.php$".to_string()];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::Regex);
        assert_eq!(path, r"\.php$");
    }

    #[test]
    fn test_location_modifier_regex_case_insensitive() {
        let args = vec!["~*".to_string(), r"\.(jpg|png|gif)$".to_string()];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::RegexCaseInsensitive);
        assert_eq!(path, r"\.(jpg|png|gif)$");
    }

    #[test]
    fn test_location_modifier_empty_args() {
        let args: Vec<String> = vec![];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::None);
        assert_eq!(path, "/");
    }

    #[test]
    fn test_location_modifier_exact_no_path() {
        let args = vec!["=".to_string()];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::Exact);
        assert_eq!(path, "/");
    }

    #[test]
    fn test_location_modifier_regex_no_pattern() {
        let args = vec!["~".to_string()];
        let (modifier, path) = LocationModifier::from_args(&args);

        assert_eq!(modifier, LocationModifier::Regex);
        assert_eq!(path, "");
    }

    #[test]
    fn test_location_with_root() {
        let mut location = Location::new("/images", LocationModifier::None);
        location.root = Some(PathBuf::from("/var/www/images"));

        assert_eq!(location.root, Some(PathBuf::from("/var/www/images")));
        assert!(location.is_static());
    }

    #[test]
    fn test_location_with_proxy_pass() {
        let mut location = Location::new("/api/v1", LocationModifier::None);
        location.proxy_pass = Some("http://api-backend:3000".to_string());

        assert_eq!(
            location.proxy_pass,
            Some("http://api-backend:3000".to_string())
        );
        assert!(location.is_proxy());
    }

    #[test]
    fn test_location_modifiers_equality() {
        assert_eq!(LocationModifier::None, LocationModifier::None);
        assert_eq!(LocationModifier::Exact, LocationModifier::Exact);
        assert_ne!(LocationModifier::None, LocationModifier::Exact);
        assert_ne!(
            LocationModifier::Regex,
            LocationModifier::RegexCaseInsensitive
        );
    }
}

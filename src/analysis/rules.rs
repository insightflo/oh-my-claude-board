//! Rule-based error analysis
//!
//! Pattern-matching engine that categorizes error messages from hook events
//! and provides retryable hints and actionable suggestions.

/// Error category derived from pattern matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    Type,
    Runtime,
    Network,
    Permission,
    Unknown,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type => write!(f, "Type"),
            Self::Runtime => write!(f, "Runtime"),
            Self::Network => write!(f, "Network"),
            Self::Permission => write!(f, "Permission"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Result of analyzing an error message
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub category: ErrorCategory,
    pub retryable: bool,
    pub suggestion: &'static str,
}

/// Rule entry: pattern to match (lowercase), category, retryable, suggestion
struct Rule {
    patterns: &'static [&'static str],
    category: ErrorCategory,
    retryable: bool,
    suggestion: &'static str,
}

const RULES: &[Rule] = &[
    // Permission
    Rule {
        patterns: &["permission denied"],
        category: ErrorCategory::Permission,
        retryable: false,
        suggestion: "Check file permissions",
    },
    Rule {
        patterns: &["access denied"],
        category: ErrorCategory::Permission,
        retryable: false,
        suggestion: "Check access rights",
    },
    // Network
    Rule {
        patterns: &["connection refused"],
        category: ErrorCategory::Network,
        retryable: true,
        suggestion: "Check if service is running",
    },
    Rule {
        patterns: &["timeout", "timed out"],
        category: ErrorCategory::Network,
        retryable: true,
        suggestion: "Retry or increase timeout",
    },
    Rule {
        patterns: &["rate limit"],
        category: ErrorCategory::Network,
        retryable: true,
        suggestion: "Wait and retry",
    },
    Rule {
        patterns: &["dns", "resolve"],
        category: ErrorCategory::Network,
        retryable: true,
        suggestion: "Check network connection",
    },
    // Type
    Rule {
        patterns: &["type error", "type mismatch"],
        category: ErrorCategory::Type,
        retryable: false,
        suggestion: "Fix type annotations",
    },
    Rule {
        patterns: &["cannot find", "not found"],
        category: ErrorCategory::Type,
        retryable: false,
        suggestion: "Check imports and paths",
    },
    Rule {
        patterns: &["undefined", "unresolved"],
        category: ErrorCategory::Type,
        retryable: false,
        suggestion: "Check variable/module names",
    },
    // Runtime
    Rule {
        patterns: &["out of memory", "oom"],
        category: ErrorCategory::Runtime,
        retryable: false,
        suggestion: "Reduce memory usage",
    },
    Rule {
        patterns: &["stack overflow"],
        category: ErrorCategory::Runtime,
        retryable: false,
        suggestion: "Check for infinite recursion",
    },
    Rule {
        patterns: &["panic", "unwrap"],
        category: ErrorCategory::Runtime,
        retryable: false,
        suggestion: "Add proper error handling",
    },
];

/// Analyze an error message and return its category, retryable hint, and suggestion.
///
/// Rules are matched in priority order (first match wins) using case-insensitive
/// substring matching.
pub fn analyze_error(message: &str) -> ErrorAnalysis {
    let lower = message.to_lowercase();

    for rule in RULES {
        if rule.patterns.iter().any(|p| lower.contains(p)) {
            return ErrorAnalysis {
                category: rule.category.clone(),
                retryable: rule.retryable,
                suggestion: rule.suggestion,
            };
        }
    }

    ErrorAnalysis {
        category: ErrorCategory::Unknown,
        retryable: false,
        suggestion: "Investigate error details",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_denied() {
        let r = analyze_error("permission denied: /etc/shadow");
        assert_eq!(r.category, ErrorCategory::Permission);
        assert!(!r.retryable);
        assert_eq!(r.suggestion, "Check file permissions");
    }

    #[test]
    fn access_denied() {
        let r = analyze_error("Access Denied for resource X");
        assert_eq!(r.category, ErrorCategory::Permission);
        assert!(!r.retryable);
    }

    #[test]
    fn connection_refused() {
        let r = analyze_error("connection refused: localhost:5432");
        assert_eq!(r.category, ErrorCategory::Network);
        assert!(r.retryable);
        assert_eq!(r.suggestion, "Check if service is running");
    }

    #[test]
    fn timeout() {
        let r = analyze_error("request timed out after 30s");
        assert_eq!(r.category, ErrorCategory::Network);
        assert!(r.retryable);
    }

    #[test]
    fn timeout_variant() {
        let r = analyze_error("connection timeout");
        assert_eq!(r.category, ErrorCategory::Network);
        assert!(r.retryable);
    }

    #[test]
    fn rate_limit() {
        let r = analyze_error("rate limit exceeded: 429");
        assert_eq!(r.category, ErrorCategory::Network);
        assert!(r.retryable);
        assert_eq!(r.suggestion, "Wait and retry");
    }

    #[test]
    fn dns_resolution() {
        let r = analyze_error("DNS lookup failed for api.example.com");
        assert_eq!(r.category, ErrorCategory::Network);
        assert!(r.retryable);
    }

    #[test]
    fn type_error() {
        let r = analyze_error("type error: expected i32 got &str");
        assert_eq!(r.category, ErrorCategory::Type);
        assert!(!r.retryable);
    }

    #[test]
    fn not_found() {
        let r = analyze_error("module 'foo' not found");
        assert_eq!(r.category, ErrorCategory::Type);
        assert!(!r.retryable);
        assert_eq!(r.suggestion, "Check imports and paths");
    }

    #[test]
    fn undefined_variable() {
        let r = analyze_error("undefined reference to 'bar'");
        assert_eq!(r.category, ErrorCategory::Type);
        assert!(!r.retryable);
    }

    #[test]
    fn out_of_memory() {
        let r = analyze_error("fatal: out of memory allocating 1GB");
        assert_eq!(r.category, ErrorCategory::Runtime);
        assert!(!r.retryable);
    }

    #[test]
    fn stack_overflow() {
        let r = analyze_error("thread 'main' has overflowed its stack overflow");
        assert_eq!(r.category, ErrorCategory::Runtime);
        assert!(!r.retryable);
    }

    #[test]
    fn panic_unwrap() {
        let r = analyze_error("thread 'main' panicked at 'called unwrap on None'");
        assert_eq!(r.category, ErrorCategory::Runtime);
        assert!(!r.retryable);
    }

    #[test]
    fn unknown_fallback() {
        let r = analyze_error("something completely unexpected happened");
        assert_eq!(r.category, ErrorCategory::Unknown);
        assert!(!r.retryable);
        assert_eq!(r.suggestion, "Investigate error details");
    }

    #[test]
    fn case_insensitive() {
        let r = analyze_error("PERMISSION DENIED");
        assert_eq!(r.category, ErrorCategory::Permission);
    }

    #[test]
    fn display_categories() {
        assert_eq!(format!("{}", ErrorCategory::Type), "Type");
        assert_eq!(format!("{}", ErrorCategory::Runtime), "Runtime");
        assert_eq!(format!("{}", ErrorCategory::Network), "Network");
        assert_eq!(format!("{}", ErrorCategory::Permission), "Permission");
        assert_eq!(format!("{}", ErrorCategory::Unknown), "Unknown");
    }

    #[test]
    fn priority_order_permission_before_not_found() {
        // "permission denied" should match Permission, not Type's "not found"
        let r = analyze_error("permission denied: file not found");
        assert_eq!(r.category, ErrorCategory::Permission);
    }

    #[test]
    fn resolve_matches_network() {
        let r = analyze_error("could not resolve host");
        assert_eq!(r.category, ErrorCategory::Network);
        assert!(r.retryable);
    }
}

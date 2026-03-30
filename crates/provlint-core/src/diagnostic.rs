use serde::{Deserialize, Serialize};

use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Schema,
    Security,
    BestPractice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub description: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub category: Category,
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>, category: Category, span: Span) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: Severity::Error,
            category,
            span,
            fix: None,
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>, category: Category, span: Span) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: Severity::Warning,
            category,
            span,
            fix: None,
        }
    }

    pub fn info(code: impl Into<String>, message: impl Into<String>, category: Category, span: Span) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: Severity::Info,
            category,
            span,
            fix: None,
        }
    }

    pub fn with_fix(mut self, description: impl Into<String>, replacement: impl Into<String>) -> Self {
        self.fix = Some(Fix {
            description: description.into(),
            replacement: replacement.into(),
        });
        self
    }
}

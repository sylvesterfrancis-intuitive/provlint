use crate::config::Format;
use crate::diagnostic::Diagnostic;

pub trait LintRule: Send + Sync {
    fn code(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn formats(&self) -> &[Format];
    fn lint(&self, content: &str, format: Format) -> Vec<Diagnostic>;
}

pub struct RuleInfo {
    pub code: &'static str,
    pub description: &'static str,
    pub formats: Vec<Format>,
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn LintRule>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }

    pub fn lint(&self, content: &str, format: Format, disabled: &[String]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for rule in &self.rules {
            if disabled.iter().any(|d| d == rule.code()) {
                continue;
            }
            if rule.formats().contains(&format) {
                diagnostics.extend(rule.lint(content, format));
            }
        }

        diagnostics.sort_by(|a, b| {
            a.span
                .start_line
                .cmp(&b.span.start_line)
                .then(a.span.start_col.cmp(&b.span.start_col))
        });

        diagnostics
    }

    pub fn rules_info(&self) -> Vec<RuleInfo> {
        self.rules
            .iter()
            .map(|r| RuleInfo {
                code: r.code(),
                description: r.description(),
                formats: r.formats().to_vec(),
            })
            .collect()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

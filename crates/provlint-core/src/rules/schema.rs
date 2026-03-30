use crate::config::Format;
use crate::diagnostic::{Category, Diagnostic};
use crate::parser::kickstart;
use crate::rule::LintRule;
use crate::span::Span;

/// SCH-006: Duplicate directives where only one is allowed.
pub struct SchDuplicateDirective;

const SINGLETON_DIRECTIVES: &[&str] = &[
    "bootloader", "clearpart", "autopart", "firewall", "selinux",
    "rootpw", "timezone", "lang", "keyboard", "zerombr",
    "ignoredisk", "text", "graphical", "cmdline", "eula",
    "url", "cdrom", "skipx",
];

impl LintRule for SchDuplicateDirective {
    fn code(&self) -> &'static str { "SCH-006" }
    fn description(&self) -> &'static str { "Duplicate directives where only one is allowed" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let parsed = kickstart::parse(content);
        let mut diagnostics = Vec::new();
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for directive in &parsed.directives {
            let name = directive.name.to_lowercase();
            if SINGLETON_DIRECTIVES.contains(&name.as_str()) {
                if let Some(first_line) = seen.get(&name) {
                    diagnostics.push(Diagnostic::warning(
                        "SCH-006",
                        format!("Duplicate '{}' directive (first seen at line {})", directive.name, first_line),
                        Category::Schema,
                        Span::line(directive.line),
                    ));
                } else {
                    seen.insert(name, directive.line);
                }
            }
        }

        diagnostics
    }
}

/// SCH-005: AutoYaST XML structure validation (beyond basic parse errors).
pub struct SchAutoYaSTStructure;

impl LintRule for SchAutoYaSTStructure {
    fn code(&self) -> &'static str { "SCH-005" }
    fn description(&self) -> &'static str { "Invalid AutoYaST XML structure" }
    fn formats(&self) -> &[Format] { &[Format::AutoYaST] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        use crate::parser::autoyast;
        let parsed = autoyast::parse(content);
        // Parser already emits SCH-005 diagnostics for XML errors
        parsed.diagnostics
    }
}

/// SCH-002/SCH-005: Autoinstall structure validation.
pub struct SchAutoinstallStructure;

impl LintRule for SchAutoinstallStructure {
    fn code(&self) -> &'static str { "SCH-002" }
    fn description(&self) -> &'static str { "Missing required fields or invalid structure in Autoinstall" }
    fn formats(&self) -> &[Format] { &[Format::Autoinstall] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        use crate::parser::autoinstall;
        let parsed = autoinstall::parse(content);
        parsed.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_duplicate_directives() {
        let content = "text\nrootpw --plaintext test1\nlang en_US\nrootpw --plaintext test2\n";
        let rule = SchDuplicateDirective;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "SCH-006");
    }
}

use crate::config::Format;
use crate::diagnostic::{Category, Diagnostic};
use crate::rule::LintRule;
use crate::span::Span;

/// BP-001: No swap partition defined.
pub struct BpNoSwap;

impl LintRule for BpNoSwap {
    fn code(&self) -> &'static str { "BP-001" }
    fn description(&self) -> &'static str { "No swap partition defined" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart, Format::Autoinstall] }

    fn lint(&self, content: &str, format: Format) -> Vec<Diagnostic> {
        match format {
            Format::Kickstart => {
                let has_swap = content.lines().any(|line| {
                    let t = line.trim();
                    (t.starts_with("logvol") || t.starts_with("part")) && t.contains("swap")
                });
                if !has_swap {
                    vec![Diagnostic::info(
                        "BP-001",
                        "No swap partition defined; consider adding a swap logvol or partition",
                        Category::BestPractice,
                        Span::line(1),
                    )]
                } else {
                    vec![]
                }
            }
            Format::Autoinstall => {
                let has_swap = content.lines().any(|line| {
                    let t = line.trim();
                    t.contains("swap:") || t.contains("fstype: swap")
                });
                if !has_swap {
                    vec![Diagnostic::info(
                        "BP-001",
                        "No swap configuration found in storage section",
                        Category::BestPractice,
                        Span::line(1),
                    )]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}

/// BP-007: No bootloader password set.
pub struct BpNoBootloaderPassword;

impl LintRule for BpNoBootloaderPassword {
    fn code(&self) -> &'static str { "BP-007" }
    fn description(&self) -> &'static str { "No bootloader password set" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let has_bootloader = content.lines().any(|line| line.trim().starts_with("bootloader"));
        let has_bl_password = content.lines().any(|line| {
            let t = line.trim();
            t.starts_with("bootloader") && (t.contains("--password=") || t.contains("--iscrypted"))
        });

        if has_bootloader && !has_bl_password {
            vec![Diagnostic::info(
                "BP-007",
                "Bootloader has no password set; consider adding --password for physical security",
                Category::BestPractice,
                Span::line(
                    content.lines()
                        .position(|l| l.trim().starts_with("bootloader"))
                        .map(|i| i + 1)
                        .unwrap_or(1),
                ),
            )]
        } else {
            vec![]
        }
    }
}

/// BP-008: Missing network hostname.
pub struct BpMissingHostname;

impl LintRule for BpMissingHostname {
    fn code(&self) -> &'static str { "BP-008" }
    fn description(&self) -> &'static str { "Missing network hostname configuration" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart, Format::Autoinstall] }

    fn lint(&self, content: &str, format: Format) -> Vec<Diagnostic> {
        match format {
            Format::Kickstart => {
                let has_hostname = content.lines().any(|line| {
                    let t = line.trim();
                    t.starts_with("network") && t.contains("--hostname=")
                });
                if !has_hostname {
                    vec![Diagnostic::info(
                        "BP-008",
                        "No hostname configured via 'network --hostname='",
                        Category::BestPractice,
                        Span::line(1),
                    )]
                } else {
                    vec![]
                }
            }
            Format::Autoinstall => {
                let has_hostname = content.lines().any(|line| {
                    let t = line.trim();
                    t.starts_with("hostname:")
                });
                if !has_hostname {
                    vec![Diagnostic::info(
                        "BP-008",
                        "No hostname configured in identity section",
                        Category::BestPractice,
                        Span::line(1),
                    )]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}

/// BP-004: Missing NTP configuration.
pub struct BpNoNtp;

impl LintRule for BpNoNtp {
    fn code(&self) -> &'static str { "BP-004" }
    fn description(&self) -> &'static str { "No NTP time source configured" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let has_ntp = content.lines().any(|line| {
            let t = line.trim();
            t.starts_with("timesource") || (t.starts_with("timezone") && t.contains("--ntpservers="))
        });

        if !has_ntp {
            vec![Diagnostic::info(
                "BP-004",
                "No NTP time source configured; consider adding timesource directives",
                Category::BestPractice,
                Span::line(1),
            )]
        } else {
            vec![]
        }
    }
}

/// BP-006: Using deprecated directives.
pub struct BpDeprecatedDirective;

const DEPRECATED: &[(&str, &str)] = &[
    ("auth", "Use 'authselect' instead of 'auth' (deprecated in RHEL 8+)"),
    ("authconfig", "Use 'authselect' instead of 'authconfig' (deprecated in RHEL 8+)"),
    ("install", "'install' directive is deprecated and implied by default"),
];

impl LintRule for BpDeprecatedDirective {
    fn code(&self) -> &'static str { "BP-006" }
    fn description(&self) -> &'static str { "Deprecated directive used" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('%') {
                continue;
            }
            let directive = trimmed.split_whitespace().next().unwrap_or("");
            for (dep, msg) in DEPRECATED {
                if directive.eq_ignore_ascii_case(dep) {
                    diagnostics.push(Diagnostic::info(
                        "BP-006",
                        *msg,
                        Category::BestPractice,
                        Span::line(idx + 1),
                    ));
                }
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_no_swap() {
        let content = "part /boot --fstype=ext4 --size=1024\npart / --fstype=xfs --size=40960\n";
        let rule = BpNoSwap;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "BP-001");
    }

    #[test]
    fn no_warning_when_swap_present() {
        let content = "logvol swap --vgname=vg00 --fstype=swap --name=swap --size=2048\n";
        let rule = BpNoSwap;
        let diags = rule.lint(content, Format::Kickstart);
        assert!(diags.is_empty());
    }

    #[test]
    fn detect_deprecated_auth() {
        let content = "auth --useshadow --passalgo=sha512\n";
        let rule = BpDeprecatedDirective;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "BP-006");
    }
}

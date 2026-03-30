use crate::config::Format;
use crate::diagnostic::{Category, Diagnostic};
use crate::rule::LintRule;
use crate::span::Span;

/// SEC-001: Plaintext root password (not --iscrypted).
pub struct SecPlaintextPassword;

impl LintRule for SecPlaintextPassword {
    fn code(&self) -> &'static str { "SEC-001" }
    fn description(&self) -> &'static str { "Plaintext root password detected" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("rootpw ") || trimmed.starts_with("rootpw\t") {
                if trimmed.contains("--plaintext") {
                    diagnostics.push(
                        Diagnostic::warning(
                            "SEC-001",
                            "Root password set with --plaintext; use --iscrypted with a hashed password instead",
                            Category::Security,
                            Span::line(idx + 1),
                        )
                        .with_fix(
                            "Use --iscrypted with a SHA-512 hash",
                            "rootpw --iscrypted <password_hash>",
                        ),
                    );
                } else if !trimmed.contains("--iscrypted") && !trimmed.contains("--lock") {
                    diagnostics.push(
                        Diagnostic::warning(
                            "SEC-001",
                            "Root password appears to be set without encryption; use --iscrypted",
                            Category::Security,
                            Span::line(idx + 1),
                        ),
                    );
                }
            }
        }

        diagnostics
    }
}

/// SEC-002: SELinux disabled.
pub struct SecSelinuxDisabled;

impl LintRule for SecSelinuxDisabled {
    fn code(&self) -> &'static str { "SEC-002" }
    fn description(&self) -> &'static str { "SELinux is disabled" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("selinux") && trimmed.contains("--disabled") {
                diagnostics.push(
                    Diagnostic::warning(
                        "SEC-002",
                        "SELinux is disabled; consider using --enforcing or --permissive",
                        Category::Security,
                        Span::line(idx + 1),
                    )
                    .with_fix(
                        "Set SELinux to enforcing",
                        "selinux --enforcing",
                    ),
                );
            }
        }

        diagnostics
    }
}

/// SEC-003: Firewall disabled.
pub struct SecFirewallDisabled;

impl LintRule for SecFirewallDisabled {
    fn code(&self) -> &'static str { "SEC-003" }
    fn description(&self) -> &'static str { "Firewall is disabled" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("firewall") && trimmed.contains("--disabled") {
                diagnostics.push(
                    Diagnostic::warning(
                        "SEC-003",
                        "Firewall is disabled; consider enabling with appropriate port rules",
                        Category::Security,
                        Span::line(idx + 1),
                    )
                    .with_fix(
                        "Enable firewall with SSH",
                        "firewall --enabled --ssh",
                    ),
                );
            }
        }

        // Also check AutoYaST firewall
        diagnostics
    }
}

/// SEC-004: SSH PermitRootLogin enabled in post-install scripts.
pub struct SecPermitRootLogin;

impl LintRule for SecPermitRootLogin {
    fn code(&self) -> &'static str { "SEC-004" }
    fn description(&self) -> &'static str { "SSH PermitRootLogin is enabled" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart, Format::Autoinstall, Format::AutoYaST] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Match: PermitRootLogin yes (in scripts, sed commands, echo statements)
            if trimmed.contains("PermitRootLogin") && trimmed.contains("yes") {
                // Skip lines that are disabling it or commenting it out
                if trimmed.starts_with('#') && !trimmed.contains("echo") && !trimmed.contains("sed") {
                    continue;
                }
                diagnostics.push(
                    Diagnostic::warning(
                        "SEC-004",
                        "SSH PermitRootLogin is set to 'yes'; consider using key-based auth only",
                        Category::Security,
                        Span::line(idx + 1),
                    ),
                );
            }
        }

        diagnostics
    }
}

/// SEC-005: Weak password hash algorithm.
pub struct SecWeakHash;

impl LintRule for SecWeakHash {
    fn code(&self) -> &'static str { "SEC-005" }
    fn description(&self) -> &'static str { "Weak password hash algorithm detected" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("rootpw") && trimmed.contains("--iscrypted") {
                // Check for MD5 hash ($1$) or SHA-256 ($5$) — prefer SHA-512 ($6$)
                if trimmed.contains("$1$") {
                    diagnostics.push(Diagnostic::warning(
                        "SEC-005",
                        "MD5 password hash detected ($1$); use SHA-512 ($6$) instead",
                        Category::Security,
                        Span::line(idx + 1),
                    ));
                } else if trimmed.contains("$5$") {
                    diagnostics.push(Diagnostic::info(
                        "SEC-005",
                        "SHA-256 password hash detected ($5$); SHA-512 ($6$) is preferred",
                        Category::Security,
                        Span::line(idx + 1),
                    ));
                }
            }
        }

        diagnostics
    }
}

/// SEC-007: Unencrypted HTTP repo URLs.
pub struct SecHttpRepo;

impl LintRule for SecHttpRepo {
    fn code(&self) -> &'static str { "SEC-007" }
    fn description(&self) -> &'static str { "Unencrypted HTTP URL used for package repository" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart, Format::AutoYaST, Format::Autoinstall] }

    fn lint(&self, content: &str, format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            match format {
                Format::Kickstart => {
                    // repo --baseurl=http:// or url --url=http://
                    if (trimmed.starts_with("repo ") || trimmed.starts_with("url "))
                        && trimmed.contains("http://")
                    {
                        diagnostics.push(Diagnostic::info(
                            "SEC-007",
                            "Repository URL uses unencrypted HTTP; consider HTTPS if available",
                            Category::Security,
                            Span::line(idx + 1),
                        ));
                    }
                }
                Format::AutoYaST => {
                    // <media_url>http://...</media_url>
                    if trimmed.contains("<media_url>") && trimmed.contains("http://") {
                        diagnostics.push(Diagnostic::info(
                            "SEC-007",
                            "Repository URL uses unencrypted HTTP; consider HTTPS if available",
                            Category::Security,
                            Span::line(idx + 1),
                        ));
                    }
                }
                Format::Autoinstall => {
                    // url: http:// in YAML
                    if trimmed.contains("http://") && (trimmed.contains("url:") || trimmed.contains("uri:")) {
                        diagnostics.push(Diagnostic::info(
                            "SEC-007",
                            "URL uses unencrypted HTTP; consider HTTPS if available",
                            Category::Security,
                            Span::line(idx + 1),
                        ));
                    }
                }
            }
        }

        diagnostics
    }
}

/// SEC-008: Empty or default passwords.
pub struct SecEmptyPassword;

impl LintRule for SecEmptyPassword {
    fn code(&self) -> &'static str { "SEC-008" }
    fn description(&self) -> &'static str { "Empty or default password detected" }
    fn formats(&self) -> &[Format] { &[Format::Kickstart] }

    fn lint(&self, content: &str, _format: Format) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // rootpw with empty string or common defaults
            if trimmed == "rootpw" || trimmed == "rootpw \"\"" || trimmed == "rootpw ''" {
                diagnostics.push(Diagnostic::error(
                    "SEC-008",
                    "Root password is empty",
                    Category::Security,
                    Span::line(idx + 1),
                ));
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_plaintext_password() {
        let content = "rootpw --plaintext mysecret\n";
        let rule = SecPlaintextPassword;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "SEC-001");
        assert!(diags[0].fix.is_some());
    }

    #[test]
    fn no_warning_for_encrypted_password() {
        let content = "rootpw --iscrypted $6$rounds=4096$salt$hash\n";
        let rule = SecPlaintextPassword;
        let diags = rule.lint(content, Format::Kickstart);
        assert!(diags.is_empty());
    }

    #[test]
    fn detect_selinux_disabled() {
        let content = "selinux --disabled\n";
        let rule = SecSelinuxDisabled;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "SEC-002");
    }

    #[test]
    fn detect_firewall_disabled() {
        let content = "firewall --disabled\n";
        let rule = SecFirewallDisabled;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "SEC-003");
    }

    #[test]
    fn detect_http_repo() {
        let content = "repo --name=test --baseurl=http://mirror.example.com/repo\n";
        let rule = SecHttpRepo;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "SEC-007");
    }

    #[test]
    fn detect_permit_root_login() {
        let content = "sed -i 's/PermitRootLogin.*/PermitRootLogin yes/' /etc/ssh/sshd_config\n";
        let rule = SecPermitRootLogin;
        let diags = rule.lint(content, Format::Kickstart);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "SEC-004");
    }
}

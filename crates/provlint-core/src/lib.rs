pub mod config;
pub mod diagnostic;
pub mod parser;
pub mod rule;
pub mod rules;
pub mod span;

use config::{Format, LintConfig};
use diagnostic::Diagnostic;
use rule::{RuleInfo, RuleRegistry};

pub struct ProvLint {
    registry: RuleRegistry,
}

impl ProvLint {
    pub fn new() -> Self {
        let mut registry = RuleRegistry::new();
        rules::register_all(&mut registry);
        Self { registry }
    }

    /// Lint content with auto-detected format.
    pub fn lint(&self, content: &str, format: Option<Format>) -> Vec<Diagnostic> {
        let format = match format {
            Some(f) => f,
            None => match Format::detect(content) {
                Some(f) => f,
                None => return vec![],
            },
        };

        self.lint_with_config(content, format, &LintConfig::default())
    }

    /// Lint content with explicit format and config.
    pub fn lint_with_config(
        &self,
        content: &str,
        format: Format,
        config: &LintConfig,
    ) -> Vec<Diagnostic> {
        // First get parser-level diagnostics
        let mut diagnostics = match format {
            Format::Kickstart => {
                let parsed = parser::kickstart::parse(content);
                parsed.diagnostics
            }
            Format::AutoYaST => vec![],  // handled by rules
            Format::Autoinstall => vec![], // handled by rules
        };

        // Then run rules
        diagnostics.extend(self.registry.lint(content, format, &config.disabled_rules));

        // Sort by line
        diagnostics.sort_by(|a, b| {
            a.span.start_line.cmp(&b.span.start_line)
                .then(a.span.start_col.cmp(&b.span.start_col))
        });

        // Deduplicate by code + line
        diagnostics.dedup_by(|a, b| {
            a.code == b.code && a.span.start_line == b.span.start_line
        });

        diagnostics
    }

    /// Detect the format of content.
    pub fn detect_format(&self, content: &str) -> Option<Format> {
        Format::detect(content)
    }

    /// List all supported rules.
    pub fn supported_rules(&self) -> Vec<RuleInfo> {
        self.registry.rules_info()
    }
}

impl Default for ProvLint {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_kickstart_end_to_end() {
        let content = r#"
#version=RHEL9
eula --agreed
reboot
text
rootpw --plaintext mysecret
selinux --disabled
firewall --disabled
bootloader --append="crashkernel=auto" --location=mbr --boot-drive=vda
clearpart --all --initlabel
part /boot --fstype="ext4" --size=1024
part pv.01 --grow --size=1
volgroup vg00 --pesize=32768 pv.01
logvol / --vgname=vg00 --fstype=xfs --size=40960 --name=root
logvol swap --vgname=vg00 --fstype=swap --name=swap --size=2048
network --hostname=testhost
timesource --ntp-server=ntp.example.com
timezone America/Los_Angeles --utc
lang en_US.UTF-8
keyboard --vckeymap=us

%packages
@core
vim
%end

%post --log=/root/ks-post.log
echo "done"
%end
"#;
        let linter = ProvLint::new();
        let diags = linter.lint(content, Some(Format::Kickstart));

        let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"SEC-001"), "Should detect plaintext password");
        assert!(codes.contains(&"SEC-002"), "Should detect SELinux disabled");
        assert!(codes.contains(&"SEC-003"), "Should detect firewall disabled");
        assert!(codes.contains(&"BP-007"), "Should detect no bootloader password");
    }

    #[test]
    fn lint_autoinstall_end_to_end() {
        let content = r#"#cloud-config
autoinstall:
  version: 1
  identity:
    hostname: testvm
    username: admin
    password: hashed
  storage:
    swap:
      size: 4G
"#;
        let linter = ProvLint::new();
        let diags = linter.lint(content, Some(Format::Autoinstall));
        // Should parse cleanly with version present
        assert!(
            !diags.iter().any(|d| d.code == "SCH-002"),
            "Should not flag missing version"
        );
    }

    #[test]
    fn format_detection() {
        let linter = ProvLint::new();

        assert_eq!(linter.detect_format("eula --agreed\ntext\n"), Some(Format::Kickstart));
        assert_eq!(linter.detect_format("<?xml version=\"1.0\"?>\n<profile>"), Some(Format::AutoYaST));
        assert_eq!(linter.detect_format("#cloud-config\nautoinstall:\n  version: 1"), Some(Format::Autoinstall));
    }

    #[test]
    fn disabled_rules() {
        let content = "rootpw --plaintext test\nselinux --disabled\n";
        let linter = ProvLint::new();
        let config = LintConfig {
            disabled_rules: vec!["SEC-001".to_string()],
        };
        let diags = linter.lint_with_config(content, Format::Kickstart, &config);
        assert!(!diags.iter().any(|d| d.code == "SEC-001"), "SEC-001 should be disabled");
        assert!(diags.iter().any(|d| d.code == "SEC-002"), "SEC-002 should still fire");
    }
}

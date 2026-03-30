use crate::diagnostic::{Category, Diagnostic};
use crate::span::Span;

/// Known Kickstart directives (commands that appear at the top level).
const KNOWN_DIRECTIVES: &[&str] = &[
    "auth", "authconfig", "authselect",
    "autopart",
    "bootloader",
    "cdrom",
    "clearpart",
    "cmdline",
    "device",
    "driverdisk",
    "eula",
    "fcoe",
    "firewall",
    "firstboot",
    "graphical",
    "group",
    "halt",
    "harddrive",
    "ignoredisk",
    "install",
    "iscsi",
    "iscsiname",
    "keyboard",
    "lang",
    "logging",
    "logvol",
    "mediacheck",
    "mount",
    "network",
    "nfs",
    "ostreesetup",
    "part", "partition",
    "poweroff",
    "raid",
    "realm",
    "reboot",
    "repo",
    "reqpart",
    "rescue",
    "rootpw",
    "selinux",
    "services",
    "shutdown",
    "skipx",
    "snapshot",
    "sshkey",
    "sshpw",
    "text",
    "timezone",
    "timesource",
    "url",
    "user",
    "vnc",
    "volgroup",
    "xconfig",
    "zerombr",
    "zfcp",
    "module",
    "syspurpose",
    "zipl",
    "liveimg",
];

/// Section markers that open a block.
const SECTION_OPENERS: &[&str] = &[
    "%packages",
    "%pre",
    "%post",
    "%addon",
    "%anaconda",
    "%onerror",
    "%traceback",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    TopLevel,
    InSection,
}

#[derive(Debug, Clone)]
pub struct KickstartSection {
    pub name: String,
    pub start_line: usize,
    pub end_line: Option<usize>,
    pub args: String,
    pub body: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KickstartDirective {
    pub name: String,
    pub args: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedKickstart {
    pub directives: Vec<KickstartDirective>,
    pub sections: Vec<KickstartSection>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(content: &str) -> ParsedKickstart {
    let mut directives = Vec::new();
    let mut sections: Vec<KickstartSection> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut state = ParserState::TopLevel;
    let mut current_section: Option<KickstartSection> = None;

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if let Some(ref mut section) = current_section {
                section.body.push(line.to_string());
            }
            continue;
        }

        // Check for %end
        if trimmed == "%end" {
            match current_section.take() {
                Some(mut section) => {
                    section.end_line = Some(line_num);
                    sections.push(section);
                    state = ParserState::TopLevel;
                }
                None => {
                    diagnostics.push(Diagnostic::error(
                        "SCH-004",
                        "%end without matching section opener",
                        Category::Schema,
                        Span::line(line_num),
                    ));
                }
            }
            continue;
        }

        // Check for section openers
        if trimmed.starts_with('%') {
            let is_section = SECTION_OPENERS.iter().any(|s| {
                trimmed == *s || trimmed.starts_with(&format!("{} ", s)) || trimmed.starts_with(&format!("{}\t", s))
            });

            if is_section {
                // Close any unclosed section
                if let Some(mut prev) = current_section.take() {
                    diagnostics.push(Diagnostic::error(
                        "SCH-004",
                        format!("Section '{}' opened at line {} was not closed with %end", prev.name, prev.start_line),
                        Category::Schema,
                        Span::line(prev.start_line),
                    ));
                    prev.end_line = Some(line_num - 1);
                    sections.push(prev);
                }

                let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
                let section_name = parts[0].to_string();
                let section_args = parts.get(1).unwrap_or(&"").to_string();

                current_section = Some(KickstartSection {
                    name: section_name,
                    start_line: line_num,
                    end_line: None,
                    args: section_args,
                    body: Vec::new(),
                });
                state = ParserState::InSection;
                continue;
            }
        }

        match state {
            ParserState::TopLevel => {
                let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
                let directive_name = parts[0];
                let directive_args = parts.get(1).unwrap_or(&"").to_string();

                let is_known = KNOWN_DIRECTIVES
                    .iter()
                    .any(|d| *d == directive_name.to_lowercase());

                if !is_known {
                    diagnostics.push(Diagnostic::warning(
                        "SCH-001",
                        format!("Unknown directive '{}'", directive_name),
                        Category::Schema,
                        Span::line_with_cols(line_num, 0, directive_name.len()),
                    ));
                }

                directives.push(KickstartDirective {
                    name: directive_name.to_string(),
                    args: directive_args,
                    line: line_num,
                });
            }
            ParserState::InSection => {
                if let Some(ref mut section) = current_section {
                    section.body.push(line.to_string());
                }
            }
        }
    }

    // Handle unclosed section at EOF
    if let Some(mut section) = current_section.take() {
        diagnostics.push(Diagnostic::error(
            "SCH-004",
            format!("Section '{}' opened at line {} was not closed with %end (reached end of file)", section.name, section.start_line),
            Category::Schema,
            Span::line(section.start_line),
        ));
        section.end_line = None;
        sections.push(section);
    }

    ParsedKickstart {
        directives,
        sections,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_kickstart() {
        let content = r#"
#version=RHEL9
eula --agreed
reboot
text
lang en_US.UTF-8

%packages
@core
vim
%end

%post --log=/root/ks-post.log
echo "done"
%end
"#;
        let result = parse(content);
        assert!(result.diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", result.diagnostics);
        assert_eq!(result.directives.len(), 4); // eula, reboot, text, lang
        assert_eq!(result.sections.len(), 2); // %packages, %post
        assert_eq!(result.sections[0].name, "%packages");
        assert_eq!(result.sections[1].name, "%post");
        assert!(result.sections[1].args.contains("--log="));
    }

    #[test]
    fn detect_unclosed_section() {
        let content = r#"
text
%packages
@core
vim
"#;
        let result = parse(content);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SCH-004");
    }

    #[test]
    fn detect_unknown_directive() {
        let content = "foobar --something\ntext\n";
        let result = parse(content);
        assert!(result.diagnostics.iter().any(|d| d.code == "SCH-001"));
    }
}

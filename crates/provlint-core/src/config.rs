use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Kickstart,
    #[serde(alias = "autoyast")]
    AutoYaST,
    Autoinstall,
}

impl Format {
    pub fn detect(content: &str) -> Option<Self> {
        let trimmed = content.trim_start();

        // AutoYaST: XML with profile root
        if trimmed.starts_with("<?xml") || trimmed.starts_with("<!DOCTYPE profile") || trimmed.starts_with("<profile") {
            return Some(Format::AutoYaST);
        }

        // Autoinstall: YAML with #cloud-config or autoinstall key
        if trimmed.starts_with("#cloud-config") {
            return Some(Format::Autoinstall);
        }
        if trimmed.starts_with("autoinstall:") {
            return Some(Format::Autoinstall);
        }

        // Kickstart: look for common directives
        let kickstart_indicators = [
            "install", "text", "graphical", "cmdline",
            "url ", "cdrom", "nfs ", "harddrive ",
            "lang ", "keyboard ", "timezone ",
            "rootpw ", "firewall ", "selinux ",
            "bootloader ", "clearpart ", "part ",
            "volgroup ", "logvol ", "autopart",
            "%packages", "%pre", "%post",
            "eula ", "repo ", "network ",
            "ignoredisk ", "zerombr",
            "#version=",
        ];

        for line in trimmed.lines().take(30) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                // Check comment-style hints
                if line.starts_with("#version=RHEL") || line.starts_with("#version=Fedora") {
                    return Some(Format::Kickstart);
                }
                continue;
            }
            for indicator in &kickstart_indicators {
                if line.starts_with(indicator) {
                    return Some(Format::Kickstart);
                }
            }
        }

        None
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ks" | "cfg" => Some(Format::Kickstart),
            "xml" => Some(Format::AutoYaST),
            "yaml" | "yml" => Some(Format::Autoinstall),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintConfig {
    #[serde(default)]
    pub disabled_rules: Vec<String>,
}

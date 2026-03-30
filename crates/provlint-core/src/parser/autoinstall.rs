use crate::diagnostic::{Category, Diagnostic};
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct ParsedAutoinstall {
    pub root: Option<serde_yml::Value>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(content: &str) -> ParsedAutoinstall {
    let mut diagnostics = Vec::new();

    // Strip #cloud-config header if present
    let yaml_content = if content.trim_start().starts_with("#cloud-config") {
        content
            .lines()
            .skip_while(|l| l.trim().starts_with("#cloud-config") || l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        content.to_string()
    };

    let value: serde_yml::Value = match serde_yml::from_str(&yaml_content) {
        Ok(v) => v,
        Err(e) => {
            // Try to extract line number from serde_yml error
            let msg = e.to_string();
            diagnostics.push(Diagnostic::error(
                "SCH-005",
                format!("YAML parse error: {}", msg),
                Category::Schema,
                Span::line(1),
            ));
            return ParsedAutoinstall {
                root: None,
                diagnostics,
            };
        }
    };

    // Validate required fields
    if let serde_yml::Value::Mapping(ref map) = value {
        // Check for autoinstall wrapper or direct keys
        let autoinstall_map = if let Some(serde_yml::Value::Mapping(inner)) =
            map.get(&serde_yml::Value::String("autoinstall".to_string()))
        {
            inner
        } else {
            map
        };

        // SCH-002: Check for required version field
        let has_version = autoinstall_map
            .get(&serde_yml::Value::String("version".to_string()))
            .is_some();

        if !has_version {
            diagnostics.push(Diagnostic::error(
                "SCH-002",
                "Missing required field 'version' in autoinstall configuration",
                Category::Schema,
                Span::line(1),
            ));
        }
    } else {
        diagnostics.push(Diagnostic::error(
            "SCH-005",
            "Autoinstall root must be a YAML mapping",
            Category::Schema,
            Span::line(1),
        ));
    }

    ParsedAutoinstall {
        root: Some(value),
        diagnostics,
    }
}

/// Look up a value by dot-separated path in a YAML mapping.
pub fn get_value<'a>(root: &'a serde_yml::Value, path: &str) -> Option<&'a serde_yml::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;

    // Handle autoinstall wrapper
    if let serde_yml::Value::Mapping(map) = current {
        if let Some(inner) = map.get(&serde_yml::Value::String("autoinstall".to_string())) {
            current = inner;
        }
    }

    for part in &parts {
        match current {
            serde_yml::Value::Mapping(map) => {
                match map.get(&serde_yml::Value::String(part.to_string())) {
                    Some(v) => current = v,
                    None => return None,
                }
            }
            _ => return None,
        }
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_autoinstall() {
        let content = r#"#cloud-config
autoinstall:
  version: 1
  identity:
    hostname: testvm
    username: admin
    password: hashed
"#;
        let result = parse(content);
        assert!(result.diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", result.diagnostics);
        assert!(result.root.is_some());
    }

    #[test]
    fn detect_missing_version() {
        let content = r#"autoinstall:
  identity:
    hostname: testvm
"#;
        let result = parse(content);
        assert!(result.diagnostics.iter().any(|d| d.code == "SCH-002"));
    }
}

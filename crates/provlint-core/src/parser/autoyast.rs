use crate::diagnostic::{Category, Diagnostic};
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct AutoYaSTElement {
    pub name: String,
    pub line: usize,
    pub children: Vec<AutoYaSTElement>,
    pub text: Option<String>,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct ParsedAutoYaST {
    pub root: Option<AutoYaSTElement>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(content: &str) -> ParsedAutoYaST {
    let mut diagnostics = Vec::new();

    // Track line numbers by byte offset
    let line_offsets = build_line_offsets(content);

    let mut reader = quick_xml::Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<AutoYaSTElement> = Vec::new();
    let mut root: Option<AutoYaSTElement> = None;
    let mut found_profile = false;

    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                let line = byte_offset_to_line(&line_offsets, reader.buffer_position());
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                let attributes: Vec<(String, String)> = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .map(|a| {
                        (
                            String::from_utf8_lossy(a.key.as_ref()).to_string(),
                            String::from_utf8_lossy(&a.value).to_string(),
                        )
                    })
                    .collect();

                if name == "profile" {
                    found_profile = true;
                }

                stack.push(AutoYaSTElement {
                    name,
                    line,
                    children: Vec::new(),
                    text: None,
                    attributes,
                });
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if let Some(element) = stack.pop() {
                    if element.name != name {
                        let line = byte_offset_to_line(&line_offsets, reader.buffer_position());
                        diagnostics.push(Diagnostic::error(
                            "SCH-005",
                            format!(
                                "Mismatched XML tags: opened '{}' at line {}, closed with '{}'",
                                element.name, element.line, name
                            ),
                            Category::Schema,
                            Span::line(line),
                        ));
                    }

                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(element);
                    } else {
                        root = Some(element);
                    }
                }
            }
            Ok(quick_xml::events::Event::Empty(ref e)) => {
                let line = byte_offset_to_line(&line_offsets, reader.buffer_position());
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                let attributes: Vec<(String, String)> = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .map(|a| {
                        (
                            String::from_utf8_lossy(a.key.as_ref()).to_string(),
                            String::from_utf8_lossy(&a.value).to_string(),
                        )
                    })
                    .collect();

                let element = AutoYaSTElement {
                    name,
                    line,
                    children: Vec::new(),
                    text: None,
                    attributes,
                };

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element);
                } else {
                    root = Some(element);
                }
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                if let Some(current) = stack.last_mut() {
                    current.text = Some(e.unescape().unwrap_or_default().to_string());
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => {
                let line = byte_offset_to_line(&line_offsets, reader.buffer_position());
                diagnostics.push(Diagnostic::error(
                    "SCH-005",
                    format!("XML parse error: {}", e),
                    Category::Schema,
                    Span::line(line),
                ));
                break;
            }
            _ => {}
        }
    }

    // Check for unclosed tags
    for unclosed in &stack {
        diagnostics.push(Diagnostic::error(
            "SCH-005",
            format!("Unclosed XML element '{}' opened at line {}", unclosed.name, unclosed.line),
            Category::Schema,
            Span::line(unclosed.line),
        ));
    }

    if !found_profile && diagnostics.is_empty() {
        diagnostics.push(Diagnostic::warning(
            "SCH-002",
            "AutoYaST profile missing <profile> root element",
            Category::Schema,
            Span::line(1),
        ));
    }

    ParsedAutoYaST { root, diagnostics }
}

fn build_line_offsets(content: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, ch) in content.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

fn byte_offset_to_line(offsets: &[usize], offset: u64) -> usize {
    let offset = offset as usize;
    match offsets.binary_search(&offset) {
        Ok(line) => line + 1,
        Err(line) => line,
    }
}

/// Recursively find an element by name.
pub fn find_element<'a>(root: &'a AutoYaSTElement, name: &str) -> Option<&'a AutoYaSTElement> {
    if root.name == name {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_element(child, name) {
            return Some(found);
        }
    }
    None
}

/// Recursively find all elements by name.
pub fn find_all_elements<'a>(root: &'a AutoYaSTElement, name: &str) -> Vec<&'a AutoYaSTElement> {
    let mut results = Vec::new();
    if root.name == name {
        results.push(root);
    }
    for child in &root.children {
        results.extend(find_all_elements(child, name));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_autoyast() {
        let content = r#"<?xml version="1.0"?>
<!DOCTYPE profile>
<profile xmlns="http://www.suse.com/1.0/yast2ns" xmlns:config="http://www.suse.com/1.0/configns">
  <general>
    <mode>
      <confirm config:type="boolean">false</confirm>
    </mode>
  </general>
</profile>"#;
        let result = parse(content);
        assert!(result.diagnostics.is_empty(), "Expected no diagnostics, got: {:?}", result.diagnostics);
        assert!(result.root.is_some());
        assert_eq!(result.root.as_ref().unwrap().name, "profile");
    }

    #[test]
    fn detect_malformed_xml() {
        let content = r#"<?xml version="1.0"?>
<profile>
  <general>
    <unclosed>
  </general>
</profile>"#;
        let result = parse(content);
        assert!(!result.diagnostics.is_empty());
    }
}

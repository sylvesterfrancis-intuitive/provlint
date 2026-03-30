use provlint_core::config::{Format, LintConfig};
use provlint_core::ProvLint;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmProvLint {
    inner: ProvLint,
}

#[derive(Serialize, Deserialize)]
struct JsDiagnostic {
    code: String,
    message: String,
    severity: String,
    category: String,
    span: JsSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<JsFix>,
}

#[derive(Serialize, Deserialize)]
struct JsSpan {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startCol")]
    start_col: usize,
    #[serde(rename = "endLine")]
    end_line: usize,
    #[serde(rename = "endCol")]
    end_col: usize,
}

#[derive(Serialize, Deserialize)]
struct JsFix {
    description: String,
    replacement: String,
}

#[derive(Serialize, Deserialize)]
struct JsRuleInfo {
    code: String,
    description: String,
    formats: Vec<String>,
}

#[derive(Deserialize)]
struct JsLintConfig {
    #[serde(default, rename = "disabledRules")]
    disabled_rules: Vec<String>,
}

fn format_from_str(s: &str) -> Option<Format> {
    match s.to_lowercase().as_str() {
        "kickstart" => Some(Format::Kickstart),
        "autoyast" => Some(Format::AutoYaST),
        "autoinstall" => Some(Format::Autoinstall),
        _ => None,
    }
}

fn format_to_str(f: Format) -> &'static str {
    match f {
        Format::Kickstart => "kickstart",
        Format::AutoYaST => "autoyast",
        Format::Autoinstall => "autoinstall",
    }
}

#[wasm_bindgen]
impl WasmProvLint {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ProvLint::new(),
        }
    }

    /// Lint content with optional format string.
    /// Returns a JSON array of diagnostics.
    pub fn lint(&self, content: &str, format: Option<String>) -> JsValue {
        let fmt = format.and_then(|f| format_from_str(&f));
        let diagnostics = self.inner.lint(content, fmt);

        let js_diags: Vec<JsDiagnostic> = diagnostics
            .into_iter()
            .map(|d| JsDiagnostic {
                code: d.code,
                message: d.message,
                severity: format!("{:?}", d.severity).to_lowercase(),
                category: match d.category {
                    provlint_core::diagnostic::Category::Schema => "schema".to_string(),
                    provlint_core::diagnostic::Category::Security => "security".to_string(),
                    provlint_core::diagnostic::Category::BestPractice => "best-practice".to_string(),
                },
                span: JsSpan {
                    start_line: d.span.start_line,
                    start_col: d.span.start_col,
                    end_line: d.span.end_line,
                    end_col: d.span.end_col,
                },
                fix: d.fix.map(|f| JsFix {
                    description: f.description,
                    replacement: f.replacement,
                }),
            })
            .collect();

        serde_wasm_bindgen::to_value(&js_diags).unwrap_or(JsValue::NULL)
    }

    /// Lint with configuration (disabled rules).
    #[wasm_bindgen(js_name = "lintWithConfig")]
    pub fn lint_with_config(&self, content: &str, format: &str, config: JsValue) -> JsValue {
        let fmt = match format_from_str(format) {
            Some(f) => f,
            None => return serde_wasm_bindgen::to_value(&Vec::<JsDiagnostic>::new()).unwrap_or(JsValue::NULL),
        };

        let js_config: JsLintConfig = serde_wasm_bindgen::from_value(config)
            .unwrap_or(JsLintConfig { disabled_rules: vec![] });

        let config = LintConfig {
            disabled_rules: js_config.disabled_rules,
        };

        let diagnostics = self.inner.lint_with_config(content, fmt, &config);

        let js_diags: Vec<JsDiagnostic> = diagnostics
            .into_iter()
            .map(|d| JsDiagnostic {
                code: d.code,
                message: d.message,
                severity: format!("{:?}", d.severity).to_lowercase(),
                category: match d.category {
                    provlint_core::diagnostic::Category::Schema => "schema".to_string(),
                    provlint_core::diagnostic::Category::Security => "security".to_string(),
                    provlint_core::diagnostic::Category::BestPractice => "best-practice".to_string(),
                },
                span: JsSpan {
                    start_line: d.span.start_line,
                    start_col: d.span.start_col,
                    end_line: d.span.end_line,
                    end_col: d.span.end_col,
                },
                fix: d.fix.map(|f| JsFix {
                    description: f.description,
                    replacement: f.replacement,
                }),
            })
            .collect();

        serde_wasm_bindgen::to_value(&js_diags).unwrap_or(JsValue::NULL)
    }

    /// Detect the format of content.
    #[wasm_bindgen(js_name = "detectFormat")]
    pub fn detect_format(&self, content: &str) -> Option<String> {
        self.inner.detect_format(content).map(|f| format_to_str(f).to_string())
    }

    /// Get all supported rules.
    #[wasm_bindgen(js_name = "getSupportedRules")]
    pub fn get_supported_rules(&self) -> JsValue {
        let rules: Vec<JsRuleInfo> = self
            .inner
            .supported_rules()
            .into_iter()
            .map(|r| JsRuleInfo {
                code: r.code.to_string(),
                description: r.description.to_string(),
                formats: r.formats.into_iter().map(|f| format_to_str(f).to_string()).collect(),
            })
            .collect();

        serde_wasm_bindgen::to_value(&rules).unwrap_or(JsValue::NULL)
    }
}

impl Default for WasmProvLint {
    fn default() -> Self {
        Self::new()
    }
}

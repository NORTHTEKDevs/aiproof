use crate::{severity::Severity, span::Span};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub category: Category,
    pub primary: Span,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fix {
    pub description: String,
    pub edits: Vec<Edit>,
    pub safe: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Edit {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Clarity,
    Security,
    Efficiency,
    Behavior,
    Portability,
    BestPractice,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_serializes_stable_json() {
        let d = Diagnostic {
            code: "AIP007".into(),
            message: "missing input boundaries".into(),
            severity: Severity::Warning,
            category: Category::Security,
            primary: Span::from_byte_range("hello", 0..5),
            labels: vec![],
            explain_url: Some("https://aiproof.dev/rules/AIP007".into()),
            fix: None,
        };
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["code"], "AIP007");
        assert_eq!(v["severity"], "warning");
        assert_eq!(v["category"], "security");
    }
}

use aiproof_core::diagnostic::Diagnostic;
use aiproof_core::severity::Severity;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct Payload<'a> {
    version: &'static str,
    diagnostics: Vec<PayloadDiag<'a>>,
    summary: Summary,
}

#[derive(Serialize)]
struct PayloadDiag<'a> {
    file: String,
    #[serde(flatten)]
    diag: &'a Diagnostic,
}

#[derive(Serialize)]
struct Summary {
    error: usize,
    warning: usize,
    info: usize,
}

pub fn render<W: Write>(report: &crate::Report, out: &mut W) -> anyhow::Result<()> {
    let mut diags = Vec::new();
    let mut summary = Summary {
        error: 0,
        warning: 0,
        info: 0,
    };

    for (path, _src, ds) in &report.file_entries {
        for d in ds {
            match d.severity {
                Severity::Error => summary.error += 1,
                Severity::Warning => summary.warning += 1,
                Severity::Info => summary.info += 1,
            }
            diags.push(PayloadDiag {
                file: path.display().to_string(),
                diag: d,
            });
        }
    }

    let payload = Payload {
        version: "0.1",
        diagnostics: diags,
        summary,
    };
    serde_json::to_writer_pretty(&mut *out, &payload)?;
    writeln!(out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiproof_core::{diagnostic::*, severity::Severity, span::Span};
    use std::path::PathBuf;

    fn sample() -> crate::Report<'static> {
        let source = "test source\n";
        let sp = Span::from_byte_range(source, 0..4);
        crate::Report {
            file_entries: vec![(
                PathBuf::from("test.md"),
                source.to_string(),
                vec![Diagnostic {
                    code: "AIP001".into(),
                    message: "test message".into(),
                    severity: Severity::Error,
                    category: Category::Clarity,
                    primary: sp,
                    labels: vec![],
                    explain_url: None,
                    fix: None,
                }],
            )],
            _phantom: Default::default(),
        }
    }

    #[test]
    fn json_renders_valid_schema() {
        let report = sample();
        let mut output = Vec::new();
        render(&report, &mut output).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["version"], "0.1");
        assert_eq!(value["diagnostics"][0]["code"], "AIP001");
        assert_eq!(value["summary"]["error"], 1);
        assert_eq!(value["summary"]["warning"], 0);
    }

    #[test]
    fn json_counts_severities() {
        let source = "test\n";
        let sp = Span::from_byte_range(source, 0..4);
        let report = crate::Report {
            file_entries: vec![(
                PathBuf::from("test.md"),
                source.to_string(),
                vec![
                    Diagnostic {
                        code: "AIP001".into(),
                        message: "error".into(),
                        severity: Severity::Error,
                        category: Category::Clarity,
                        primary: sp.clone(),
                        labels: vec![],
                        explain_url: None,
                        fix: None,
                    },
                    Diagnostic {
                        code: "AIP002".into(),
                        message: "warning".into(),
                        severity: Severity::Warning,
                        category: Category::Clarity,
                        primary: sp.clone(),
                        labels: vec![],
                        explain_url: None,
                        fix: None,
                    },
                    Diagnostic {
                        code: "AIP003".into(),
                        message: "info".into(),
                        severity: Severity::Info,
                        category: Category::Clarity,
                        primary: sp,
                        labels: vec![],
                        explain_url: None,
                        fix: None,
                    },
                ],
            )],
            _phantom: Default::default(),
        };
        let mut output = Vec::new();
        render(&report, &mut output).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["summary"]["error"], 1);
        assert_eq!(value["summary"]["warning"], 1);
        assert_eq!(value["summary"]["info"], 1);
    }
}

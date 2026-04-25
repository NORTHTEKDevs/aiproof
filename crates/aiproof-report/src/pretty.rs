use aiproof_core::severity::Severity as AiSev;
use codespan_reporting::diagnostic::{Diagnostic as CsDiag, Label, LabelStyle, Severity as CsSev};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::NoColor;
use std::io::Write;

pub fn render<W: Write>(
    report: &crate::Report,
    _color: crate::Color,
    out: &mut W,
) -> anyhow::Result<()> {
    let config = term::Config::default();

    for (path, source, diags) in &report.file_entries {
        let file = SimpleFile::new(path.display().to_string(), source.as_str());

        for d in diags {
            let severity = match d.severity {
                AiSev::Error => CsSev::Error,
                AiSev::Warning => CsSev::Warning,
                AiSev::Info => CsSev::Note,
            };

            let mut labels = vec![
                Label::new(LabelStyle::Primary, (), d.primary.byte_range.clone())
                    .with_message(format!("in rule {}", d.code)),
            ];

            for label in &d.labels {
                labels.push(
                    Label::new(LabelStyle::Secondary, (), label.span.byte_range.clone())
                        .with_message(&label.message),
                );
            }

            let mut cs_diag = CsDiag::new(severity)
                .with_code(&d.code)
                .with_message(&d.message)
                .with_labels(labels);

            if let Some(url) = &d.explain_url {
                cs_diag = cs_diag.with_notes(vec![format!("See: {}", url)]);
            }

            let mut writer = NoColor::new(Vec::new());
            term::emit(&mut writer, &config, &file, &cs_diag)?;

            let buf = writer.into_inner();
            out.write_all(&buf)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiproof_core::{diagnostic::*, severity::Severity, span::Span};
    use std::path::PathBuf;

    fn sample() -> crate::Report<'static> {
        let source = "system prompt with sk-ant-api03-abcdefghijklmnopqrstuvwxyz\n";
        let sp = Span::from_byte_range(source, 20..63);
        crate::Report {
            file_entries: vec![(
                PathBuf::from("prompt.md"),
                source.to_string(),
                vec![Diagnostic {
                    code: "AIP006".into(),
                    message: "hardcoded credential in prompt text".into(),
                    severity: Severity::Error,
                    category: Category::Security,
                    primary: sp,
                    labels: vec![],
                    explain_url: Some(
                        "https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP006.md"
                            .into(),
                    ),
                    fix: None,
                }],
            )],
            _phantom: Default::default(),
        }
    }

    #[test]
    fn pretty_renders_without_panic() {
        let report = sample();
        let mut output = Vec::new();
        render(&report, crate::Color::Never, &mut output).unwrap();
        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("AIP006"));
        assert!(s.contains("hardcoded credential"));
    }
}

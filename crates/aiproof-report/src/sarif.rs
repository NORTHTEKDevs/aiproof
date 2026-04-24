use aiproof_core::severity::Severity;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Serialize)]
struct SarifOutput {
    version: String,
    #[serde(rename = "$schema")]
    schema: String,
    runs: Vec<Run>,
}

#[derive(Serialize)]
struct Run {
    tool: Tool,
    results: Vec<Result>,
}

#[derive(Serialize)]
struct Tool {
    driver: Driver,
}

#[derive(Serialize)]
struct Driver {
    name: String,
    version: String,
    #[serde(rename = "informationUri")]
    information_uri: String,
    rules: Vec<Rule>,
}

#[derive(Serialize)]
struct Rule {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: Description,
}

#[derive(Serialize)]
struct Description {
    text: String,
}

#[derive(Serialize)]
struct Result {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: Message,
    locations: Vec<Location>,
}

#[derive(Serialize)]
struct Message {
    text: String,
}

#[derive(Serialize)]
struct Location {
    #[serde(rename = "physicalLocation")]
    physical_location: PhysicalLocation,
}

#[derive(Serialize)]
struct PhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: ArtifactLocation,
    region: Region,
}

#[derive(Serialize)]
struct ArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct Region {
    #[serde(rename = "startLine")]
    start_line: u32,
    #[serde(rename = "startColumn")]
    start_column: u32,
    #[serde(rename = "endLine")]
    end_line: u32,
    #[serde(rename = "endColumn")]
    end_column: u32,
}

pub fn render<W: Write>(report: &crate::Report, out: &mut W) -> anyhow::Result<()> {
    let mut all_results = Vec::new();
    let mut rule_set: BTreeMap<String, String> = BTreeMap::new();

    for (path, _src, diags) in &report.file_entries {
        for d in diags {
            rule_set.insert(d.code.clone(), d.message.clone());

            let level = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };

            all_results.push(Result {
                rule_id: d.code.clone(),
                level: level.to_string(),
                message: Message {
                    text: d.message.clone(),
                },
                locations: vec![Location {
                    physical_location: PhysicalLocation {
                        artifact_location: ArtifactLocation {
                            uri: path.display().to_string(),
                        },
                        region: Region {
                            start_line: d.primary.start_line,
                            start_column: d.primary.start_col,
                            end_line: d.primary.end_line,
                            end_column: d.primary.end_col,
                        },
                    },
                }],
            });
        }
    }

    let rules: Vec<Rule> = rule_set
        .into_iter()
        .map(|(id, msg)| Rule {
            id: id.clone(),
            name: id,
            short_description: Description { text: msg },
        })
        .collect();

    let output = SarifOutput {
        version: "2.1.0".to_string(),
        schema: "https://json.schemastore.org/sarif-2.1.0.json".to_string(),
        runs: vec![Run {
            tool: Tool {
                driver: Driver {
                    name: "aiproof".to_string(),
                    version: "0.1.0".to_string(),
                    information_uri: "https://github.com/northtek/aiproof".to_string(),
                    rules,
                },
            },
            results: all_results,
        }],
    };

    serde_json::to_writer_pretty(&mut *out, &output)?;
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
                    code: "AIP006".into(),
                    message: "security issue".into(),
                    severity: Severity::Error,
                    category: Category::Security,
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
    fn sarif_renders_valid_structure() {
        let report = sample();
        let mut output = Vec::new();
        render(&report, &mut output).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["version"], "2.1.0");
        assert_eq!(
            value["$schema"],
            "https://json.schemastore.org/sarif-2.1.0.json"
        );
        assert_eq!(value["runs"][0]["tool"]["driver"]["name"], "aiproof");
    }

    #[test]
    fn sarif_includes_result_with_location() {
        let report = sample();
        let mut output = Vec::new();
        render(&report, &mut output).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["runs"][0]["results"][0]["ruleId"], "AIP006");
        assert_eq!(value["runs"][0]["results"][0]["level"], "error");
        assert_eq!(
            value["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["artifactLocation"]
                ["uri"],
            "test.md"
        );
    }
}

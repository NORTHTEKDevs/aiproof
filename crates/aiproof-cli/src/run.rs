use aiproof_config::Config;
use aiproof_core::rule::{Ctx, Rule};
use aiproof_core::severity::Severity;
use aiproof_report::{Color, Format, Report};
use std::path::PathBuf;

pub struct RunArgs<'a> {
    pub paths: &'a [PathBuf],
    pub config: Config,
    pub format: crate::cli::Format,
    pub color: Color,
}

pub fn run(args: RunArgs) -> anyhow::Result<i32> {
    let files = crate::discovery::discover(args.paths, &args.config)?;
    let rules = filtered_rules(&args.config);
    let ctx = Ctx {
        target_models: args.config.target_models.as_slice(),
        max_tokens_budget: args.config.max_tokens_budget,
    };
    let mut entries = Vec::new();
    let mut max_severity: Option<Severity> = None;

    for f in files {
        let Ok(source) = std::fs::read_to_string(&f.path) else {
            continue;
        };
        let Ok(docs) = aiproof_parse::parse_file(&f.path, &source) else {
            continue;
        };
        let mut file_diags: Vec<_> = Vec::new();
        for doc in &docs {
            for rule in &rules {
                for d in rule.check(doc, &ctx) {
                    max_severity = Some(match max_severity {
                        Some(cur) if cur >= d.severity => cur,
                        _ => d.severity,
                    });
                    file_diags.push(d);
                }
            }
        }
        if !file_diags.is_empty() {
            entries.push((f.path.clone(), source, file_diags));
        }
    }

    let report = Report {
        file_entries: entries,
        _phantom: Default::default(),
    };
    let format = match args.format {
        crate::cli::Format::Pretty => Format::Pretty,
        crate::cli::Format::Json => Format::Json,
        crate::cli::Format::Sarif => Format::Sarif,
    };
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    aiproof_report::render(&report, format, args.color, &mut out)?;

    Ok(max_severity.map(|s| s.exit_code()).unwrap_or(0))
}

fn filtered_rules(config: &Config) -> Vec<Box<dyn Rule>> {
    let all = aiproof_rules::all_rules();
    all.into_iter()
        .filter(|r| {
            let code = r.code();
            if !config.select.is_empty() && !config.select.iter().any(|s| code_matches(s, code)) {
                return false;
            }
            if config.ignore.iter().any(|s| code_matches(s, code)) {
                return false;
            }
            true
        })
        .collect()
}

fn code_matches(pattern: &str, code: &str) -> bool {
    // Support "AIP*" wildcard and exact match.
    if let Some(prefix) = pattern.strip_suffix('*') {
        code.starts_with(prefix)
    } else {
        code == pattern
    }
}

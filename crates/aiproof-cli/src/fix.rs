/// Apply safe autofixes idempotently.
use aiproof_config::Config;
use aiproof_core::rule::Ctx;
use std::path::PathBuf;

pub struct FixOutcome {
    pub fixes_applied: usize,
    pub files_changed: usize,
}

pub fn run_fix(
    paths: &[PathBuf],
    config: &Config,
    unsafe_fixes: bool,
) -> anyhow::Result<FixOutcome> {
    let files = crate::discovery::discover(paths, config)?;
    let rules = crate::run::filtered_rules(config);
    let ctx = Ctx {
        target_models: config.target_models.as_slice(),
        max_tokens_budget: config.max_tokens_budget,
    };

    let mut fixes_applied = 0;
    let mut files_changed = 0;

    for f in files {
        if std::fs::metadata(&f.path).is_ok_and(|m| m.len() > 10 * 1024 * 1024) {
            continue;
        }
        let Ok(mut source) = std::fs::read_to_string(&f.path) else {
            continue;
        };
        let original = source.clone();

        // Iterate until no more fixes. Cap at 10 iterations.
        for _ in 0..10 {
            let docs = match aiproof_parse::parse_file(&f.path, &source) {
                Ok(d) => d,
                Err(_) => break,
            };
            let mut edits_for_pass = Vec::new();
            for doc in &docs {
                for rule in &rules {
                    for diag in rule.check(doc, &ctx) {
                        if let Some(fix) = rule.autofix(&diag, doc) {
                            if !unsafe_fixes && !fix.safe {
                                continue;
                            }
                            edits_for_pass.extend(fix.edits);
                        }
                    }
                }
            }
            if edits_for_pass.is_empty() {
                break;
            }

            // Apply edits in reverse-byte-position order to avoid shifting offsets.
            // Skip any edit that overlaps with a previously-applied (later-positioned) one
            // OR that lands on a non-char boundary in the (possibly UTF-8) source.
            edits_for_pass.sort_by_key(|e| std::cmp::Reverse(e.span.byte_range.start));
            let mut next_safe_end = source.len();
            for e in &edits_for_pass {
                let range = e.span.byte_range.clone();
                if range.end > source.len() || range.start > range.end {
                    continue;
                }
                // Reject overlap: the previous (higher-start) edit covered this region.
                if range.end > next_safe_end {
                    continue;
                }
                // Reject if either endpoint is not on a UTF-8 char boundary.
                if !source.is_char_boundary(range.start) || !source.is_char_boundary(range.end) {
                    continue;
                }
                source.replace_range(range.clone(), &e.replacement);
                next_safe_end = range.start;
                fixes_applied += 1;
            }
        }

        if source != original {
            // Atomic rewrite: write to a tempfile in the same dir, then rename.
            let tmp = f.path.with_extension("aiproof-tmp");
            std::fs::write(&tmp, &source)?;
            std::fs::rename(&tmp, &f.path)?;
            files_changed += 1;
        }
    }

    Ok(FixOutcome {
        fixes_applied,
        files_changed,
    })
}

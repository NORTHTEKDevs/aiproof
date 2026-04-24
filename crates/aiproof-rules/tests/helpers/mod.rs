//! Shared test helpers for rule snapshot tests.
//!
//! Each rule test file imports `helpers::run_rule` and feeds it a source string
//! plus the file extension (so the right parser runs). Returns all diagnostics
//! produced by the rule across all extracted documents.

use aiproof_core::{
    diagnostic::Diagnostic,
    rule::{Ctx, Rule},
};
use std::path::PathBuf;

pub fn run_rule<R: Rule>(rule: R, src: &str, ext: &str) -> Vec<Diagnostic> {
    run_rule_with_ctx(
        rule,
        src,
        ext,
        Ctx {
            target_models: &[],
            max_tokens_budget: None,
        },
    )
}

pub fn run_rule_with_ctx<R: Rule>(rule: R, src: &str, ext: &str, ctx: Ctx) -> Vec<Diagnostic> {
    let path = PathBuf::from(format!("test.{ext}"));
    let docs = aiproof_parse::parse_file(&path, src).expect("parse_file should succeed in tests");
    docs.iter().flat_map(|d| rule.check(d, &ctx)).collect()
}

#[allow(dead_code)]
pub fn run_rule_with_models<R: Rule>(
    rule: R,
    src: &str,
    ext: &str,
    target_models: &[String],
) -> Vec<Diagnostic> {
    run_rule_with_ctx(
        rule,
        src,
        ext,
        Ctx {
            target_models,
            max_tokens_budget: None,
        },
    )
}

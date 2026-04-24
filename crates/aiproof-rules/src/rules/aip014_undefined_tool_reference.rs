//! AIP014: undefined-tool-reference
//!
//! Detect references to tools in the prompt that are not defined in a known registry.
//! v0: stub implementation. Registry discovery deferred to v0.1.

use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};

pub struct UndefinedToolReference;

impl Rule for UndefinedToolReference {
    fn code(&self) -> &'static str {
        "AIP014"
    }

    fn name(&self) -> &'static str {
        "undefined-tool-reference"
    }

    fn category(&self) -> Category {
        Category::Behavior
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, _doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        // TODO: wire tool registry discovery in v0.1
        // For now, return no diagnostics to avoid false positives.
        // Future: scan for patterns like "call the <name> tool", "use the <name> function",
        // "invoke <name>", "@<name>"; check against tools.json/tools.yaml in sibling directory.
        Vec::new()
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(UndefinedToolReference));
}

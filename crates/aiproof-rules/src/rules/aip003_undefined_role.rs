//! AIP003: undefined-role
//!
//! Detect conflicting role declarations in the prompt.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};
use regex::Regex;
use std::collections::HashSet;

pub struct UndefinedRole;

impl Rule for UndefinedRole {
    fn code(&self) -> &'static str {
        "AIP003"
    }

    fn name(&self) -> &'static str {
        "undefined-role"
    }

    fn category(&self) -> Category {
        Category::Clarity
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text = &doc.prompt.text;
        let mut roles = Vec::new();
        let mut distinct_roles = HashSet::new();

        // Find all role matches with their byte positions
        if let Ok(re) = Regex::new(r"(?i)you are (?:a |an )?([a-z ]+?)(?:\s*[.,!?\n]|$)") {
            for cap in re.captures_iter(text) {
                if let Some(role_match) = cap.get(1) {
                    let role = role_match.as_str().trim().to_lowercase();
                    let absolute_pos = role_match.start();
                    roles.push((role.clone(), absolute_pos, role_match.len()));
                    distinct_roles.insert(role);
                }
            }
        }

        let mut diags = Vec::new();

        // Only flag if we have 2+ distinct roles
        if distinct_roles.len() >= 2 && roles.len() >= 2 {
            // Find the second occurrence in the original list and emit at that position
            let (role2, pos2, len2) = &roles[1];
            let span = project_span(doc, *pos2..*pos2 + *len2);
            let role1 = &roles[0].0;

            diags.push(Diagnostic {
                code: "AIP003".to_string(),
                message: format!(
                    "conflicting role declarations: \"{}\" then \"{}\"",
                    role1, role2
                ),
                severity: Severity::Info,
                category: Category::Clarity,
                primary: span,
                labels: vec![],
                explain_url: Some("https://aiproof.dev/rules/AIP003".to_string()),
                fix: None,
            });
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(UndefinedRole));
}

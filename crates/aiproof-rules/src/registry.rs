use aiproof_core::rule::Rule;

/// Return the full v0 rule set. New rules register themselves in `crate::rules::register_all`.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut v: Vec<Box<dyn Rule>> = Vec::new();
    crate::rules::register_all(&mut v);
    v
}

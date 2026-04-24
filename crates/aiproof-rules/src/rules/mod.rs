use aiproof_core::rule::Rule;

// Individual rule modules are added by Phase 4 tasks (AIP001..AIP020).
// Each rule module defines a struct + `impl Rule` + a register function.
//
// Example (added later):
//   pub mod AIP006_hardcoded_credential;
//   AIP006_hardcoded_credential::register(out);

/// Called by `crate::registry::all_rules` to populate the rule vector.
pub fn register_all(out: &mut Vec<Box<dyn Rule>>) {
    let _ = out; // placeholder until rules arrive
}

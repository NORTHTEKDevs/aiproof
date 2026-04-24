use aiproof_core::rule::Rule;

pub mod aip001_conflicting_instructions;
pub mod aip002_ambiguous_output_format;
pub mod aip003_undefined_role;
pub mod aip004_contradictory_tone;

/// Called by `crate::registry::all_rules` to populate the rule vector.
pub fn register_all(out: &mut Vec<Box<dyn Rule>>) {
    aip001_conflicting_instructions::register(out);
    aip002_ambiguous_output_format::register(out);
    aip003_undefined_role::register(out);
    aip004_contradictory_tone::register(out);
}

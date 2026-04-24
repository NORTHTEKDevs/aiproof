use aiproof_core::rule::Rule;

pub mod aip001_conflicting_instructions;
pub mod aip002_ambiguous_output_format;
pub mod aip003_undefined_role;
pub mod aip004_contradictory_tone;
pub mod aip005_unescaped_user_input;
pub mod aip006_hardcoded_credential;
pub mod aip007_missing_input_boundaries;
pub mod aip008_known_jailbreak_pattern;

/// Called by `crate::registry::all_rules` to populate the rule vector.
pub fn register_all(out: &mut Vec<Box<dyn Rule>>) {
    aip001_conflicting_instructions::register(out);
    aip002_ambiguous_output_format::register(out);
    aip003_undefined_role::register(out);
    aip004_contradictory_tone::register(out);
    aip005_unescaped_user_input::register(out);
    aip006_hardcoded_credential::register(out);
    aip007_missing_input_boundaries::register(out);
    aip008_known_jailbreak_pattern::register(out);
}

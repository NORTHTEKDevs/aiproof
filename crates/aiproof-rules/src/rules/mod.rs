use aiproof_core::rule::Rule;

pub mod aip001_conflicting_instructions;
pub mod aip002_ambiguous_output_format;
pub mod aip003_undefined_role;
pub mod aip004_contradictory_tone;
pub mod aip005_unescaped_user_input;
pub mod aip006_hardcoded_credential;
pub mod aip007_missing_input_boundaries;
pub mod aip008_known_jailbreak_pattern;
pub mod aip009_cache_unfriendly_structure;
pub mod aip010_redundant_instruction;
pub mod aip011_excessive_tokens;
pub mod aip012_unused_template_variable;
pub mod aip013_missing_format_example;
pub mod aip014_undefined_tool_reference;
pub mod aip015_unhandled_placeholder;
pub mod aip016_claude_specific_tags_on_gpt;
pub mod aip017_system_message_mismatch;
pub mod aip018_temperature_determinism_mismatch;
pub mod aip019_missing_few_shot_for_reasoning;
pub mod aip020_system_message_overloaded;

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
    aip009_cache_unfriendly_structure::register(out);
    aip010_redundant_instruction::register(out);
    aip011_excessive_tokens::register(out);
    aip012_unused_template_variable::register(out);
    aip013_missing_format_example::register(out);
    aip014_undefined_tool_reference::register(out);
    aip015_unhandled_placeholder::register(out);
    aip016_claude_specific_tags_on_gpt::register(out);
    aip017_system_message_mismatch::register(out);
    aip018_temperature_determinism_mismatch::register(out);
    aip019_missing_few_shot_for_reasoning::register(out);
    aip020_system_message_overloaded::register(out);
}

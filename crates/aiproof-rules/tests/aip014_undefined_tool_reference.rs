mod helpers;

use aiproof_rules::rules::aip014_undefined_tool_reference::UndefinedToolReference;
use helpers::run_rule;

#[test]
fn v0_stub_returns_no_diagnostics() {
    // v0: always returns zero diagnostics.
    // TODO: add tool registry discovery in v0.1.
    let src = "Call the weather_tool function. Use the database_query tool.";
    let diags = run_rule(UndefinedToolReference, src, "md");
    assert!(diags.is_empty());
}

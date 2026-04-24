#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub select: Vec<String>,
    pub ignore: Vec<String>,
    pub target_models: Vec<String>,
    pub max_tokens_budget: Option<usize>,
    pub fix: bool,
    pub unsafe_fixes: bool,
}

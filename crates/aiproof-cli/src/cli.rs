use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "aiproof", version, about = "Static analyzer for AI prompts")]
pub struct Cli {
    /// Paths to scan. Defaults to current directory.
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Pretty)]
    pub format: Format,

    /// Enable specific rule codes (overrides config).
    #[arg(long)]
    pub select: Vec<String>,

    /// Disable specific rule codes (overrides config).
    #[arg(long)]
    pub ignore: Vec<String>,

    /// Target model hint for portability rules (repeatable).
    #[arg(long = "target-model")]
    pub target_models: Vec<String>,

    /// Apply safe autofixes to files in place. Idempotent.
    #[arg(long)]
    pub fix: bool,

    /// Also apply autofixes the rule author marked unsafe.
    #[arg(long, requires = "fix")]
    pub unsafe_fixes: bool,

    /// Color control.
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    pub color: ColorMode,

    /// Print the bundled explanation for a rule code and exit.
    #[arg(long, value_name = "CODE")]
    pub explain: Option<String>,

    /// Print a starter config + pre-commit snippet and exit.
    #[arg(long)]
    pub init: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Pretty,
    Json,
    Sarif,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

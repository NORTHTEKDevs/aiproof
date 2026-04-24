use aiproof_cli::cli::{Cli, ColorMode};
use aiproof_cli::run::{RunArgs, run};
use clap::Parser;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Some(code) = cli.explain.as_deref() {
        std::process::exit(aiproof_cli::explain::run_explain(code));
    }

    if cli.init {
        std::process::exit(aiproof_cli::init_snippets::run_init());
    }

    if cli.fix {
        // Load config anchored at cwd for fix command.
        let anchor = cli.paths.first().cloned().unwrap_or_else(|| ".".into());
        let config = match aiproof_config::load(&anchor) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("aiproof: config error — {e}");
                std::process::exit(2);
            }
        };
        match aiproof_cli::fix::run_fix(&cli.paths, &config, cli.unsafe_fixes) {
            Ok(outcome) => {
                println!(
                    "aiproof --fix: {} fixes applied across {} files",
                    outcome.fixes_applied, outcome.files_changed
                );
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("aiproof: --fix error: {e:?}");
                std::process::exit(2);
            }
        }
    }

    // Load config anchored at cwd (first path arg is the primary hint).
    let anchor = cli.paths.first().cloned().unwrap_or_else(|| ".".into());
    let mut config = match aiproof_config::load(&anchor) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("aiproof: config error — {e}");
            std::process::exit(2);
        }
    };

    // CLI overrides.
    if !cli.select.is_empty() {
        config.select = cli.select.clone();
    }
    if !cli.ignore.is_empty() {
        config.ignore = cli.ignore.clone();
    }
    if !cli.target_models.is_empty() {
        config.target_models = cli.target_models.clone();
    }

    let color = match cli.color {
        ColorMode::Auto => aiproof_report::Color::Auto,
        ColorMode::Always => aiproof_report::Color::Always,
        ColorMode::Never => aiproof_report::Color::Never,
    };

    let exit = match run(RunArgs {
        paths: &cli.paths,
        config,
        format: cli.format,
        color,
    }) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("aiproof: {e:?}");
            2
        }
    };

    std::process::exit(exit);
}

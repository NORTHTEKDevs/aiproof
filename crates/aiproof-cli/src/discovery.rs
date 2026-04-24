use aiproof_config::Config;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct Discovered {
    pub path: PathBuf,
}

pub fn discover(paths: &[PathBuf], config: &Config) -> anyhow::Result<Vec<Discovered>> {
    let mut out = Vec::new();
    for root in paths {
        for result in WalkBuilder::new(root).build() {
            let entry = result?;
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let path = entry.into_path();
            if !should_scan(&path, config) {
                continue;
            }
            out.push(Discovered { path });
        }
    }
    Ok(out)
}

fn should_scan(path: &Path, config: &Config) -> bool {
    // Tier 1: declared `include` globs — if set, require a match.
    if !config.include.is_empty() {
        let s = path.to_string_lossy();
        if !config.include.iter().any(|pat| glob_match(pat, &s)) {
            return false;
        }
    }
    // `exclude` always applies.
    let s = path.to_string_lossy();
    if config.exclude.iter().any(|pat| glob_match(pat, &s)) {
        return false;
    }

    // Tier 2: known-safe extensions/dirs.
    if is_known_prompt_file(path) {
        return true;
    }

    // Tier 3: .py/.ts/.tsx for SDK extraction.
    if matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("py" | "ts" | "tsx")
    ) {
        return true;
    }

    // If include globs matched (tier 1 positive), we already returned early above —
    // so if we're here with no include list, apply default extensions only.
    false
}

fn is_known_prompt_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // .prompt.md / .prompt
    if name.ends_with(".prompt.md") || name.ends_with(".prompt") {
        return true;
    }
    // Jinja-ish extensions
    if matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("j2" | "jinja" | "jinja2" | "mustache")
    ) {
        return true;
    }
    // Markdown files
    if matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md" | "yaml" | "yml")
    ) {
        return true;
    }
    // Directories named prompts, templates, system_prompts
    let components: Vec<_> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    components
        .iter()
        .any(|c| matches!(*c, "prompts" | "templates" | "system_prompts"))
}

fn glob_match(pattern: &str, path: &str) -> bool {
    glob::Pattern::new(pattern)
        .map(|p| p.matches(path))
        .unwrap_or(false)
}

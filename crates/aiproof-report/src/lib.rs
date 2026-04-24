//! aiproof-report: Three renderers for diagnostics (pretty, JSON, SARIF).

use aiproof_core::diagnostic::Diagnostic;
use std::io::Write;
use std::path::PathBuf;

pub mod json;
pub mod pretty;
pub mod sarif;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Pretty,
    Json,
    Sarif,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Auto,
    Always,
    Never,
}

pub struct Report<'a> {
    pub file_entries: Vec<(PathBuf, String, Vec<Diagnostic>)>,
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

pub fn render<W: Write>(
    report: &Report,
    format: Format,
    color: Color,
    out: &mut W,
) -> anyhow::Result<()> {
    match format {
        Format::Pretty => pretty::render(report, color, out),
        Format::Json => json::render(report, out),
        Format::Sarif => sarif::render(report, out),
    }
}

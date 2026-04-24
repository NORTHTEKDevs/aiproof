//! Shared helpers for rule implementations.

use aiproof_core::{
    document::{Document, Kind, Role},
    span::Span,
};

/// Convert a local byte range within `doc.prompt.text` into a Span in the full source.
pub fn project_span(doc: &Document, local_range: std::ops::Range<usize>) -> Span {
    let base = doc
        .prompt
        .origin_span
        .as_ref()
        .map(|s| s.byte_range.start)
        .unwrap_or(0);
    let absolute = (base + local_range.start)..(base + local_range.end);
    Span::from_byte_range(&doc.source, absolute)
}

/// Return true when the document is unambiguously a prompt — an explicit role,
/// an SDK-extracted string, a Prompty YAML, or a `.prompt.md` / `.prompt` file.
/// Rules that over-fire on README-style markdown should gate on this.
pub fn is_prompt_shaped(doc: &Document) -> bool {
    if !matches!(doc.role, Role::Unknown) {
        return true;
    }
    if matches!(
        doc.kind,
        Kind::ExtractedPython { .. }
            | Kind::ExtractedTypeScript { .. }
            | Kind::Jinja { .. }
            | Kind::Mustache { .. }
    ) {
        return true;
    }
    let name = doc.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with(".prompt.md") || name.ends_with(".prompt")
}

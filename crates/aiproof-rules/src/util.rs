//! Shared helpers for rule implementations.

use aiproof_core::{document::Document, span::Span};

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

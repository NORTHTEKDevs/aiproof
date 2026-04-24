// TODO: real implementation in Phase 2b
use aiproof_core::document::Document;
use std::path::Path;

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    crate::plain::parse(path, source)
}

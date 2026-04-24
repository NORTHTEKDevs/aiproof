use crate::{
    diagnostic::{Category, Diagnostic, Fix},
    document::Document,
    severity::Severity,
};

pub struct Ctx<'a> {
    pub target_models: &'a [String],
    pub max_tokens_budget: Option<usize>,
}

pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> Category;
    fn severity(&self) -> Severity;
    fn check(&self, doc: &Document, ctx: &Ctx) -> Vec<Diagnostic>;

    fn autofix(&self, diag: &Diagnostic, doc: &Document) -> Option<Fix> {
        let _ = (diag, doc);
        None
    }
}

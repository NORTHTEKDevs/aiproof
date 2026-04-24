use pyo3::prelude::*;
use pyo3::types::PyDict;
use aiproof_core::rule::Ctx;

#[pyfunction]
#[pyo3(signature = (source, path = "prompt.md", target_models = None, max_tokens_budget = None))]
fn check(
    py: Python<'_>,
    source: &str,
    path: &str,
    target_models: Option<Vec<String>>,
    max_tokens_budget: Option<usize>,
) -> PyResult<Vec<Py<PyDict>>> {
    let pbuf = std::path::PathBuf::from(path);
    let docs = aiproof_parse::parse_file(&pbuf, source)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("parse error: {e}")))?;
    let rules = aiproof_rules::all_rules();
    let empty: Vec<String> = Vec::new();
    let models = target_models.as_ref().unwrap_or(&empty);
    let ctx = Ctx { target_models: models.as_slice(), max_tokens_budget };

    let mut out = Vec::new();
    for doc in &docs {
        for rule in &rules {
            for d in rule.check(doc, &ctx) {
                let dict = PyDict::new_bound(py);
                dict.set_item("code", &d.code)?;
                dict.set_item("message", &d.message)?;
                dict.set_item("severity", match d.severity {
                    aiproof_core::severity::Severity::Info => "info",
                    aiproof_core::severity::Severity::Warning => "warning",
                    aiproof_core::severity::Severity::Error => "error",
                })?;
                let cat = match d.category {
                    aiproof_core::diagnostic::Category::Clarity => "clarity",
                    aiproof_core::diagnostic::Category::Security => "security",
                    aiproof_core::diagnostic::Category::Efficiency => "efficiency",
                    aiproof_core::diagnostic::Category::Behavior => "behavior",
                    aiproof_core::diagnostic::Category::Portability => "portability",
                    aiproof_core::diagnostic::Category::BestPractice => "best-practice",
                };
                dict.set_item("category", cat)?;
                dict.set_item("start_line", d.primary.start_line)?;
                dict.set_item("start_col", d.primary.start_col)?;
                dict.set_item("end_line", d.primary.end_line)?;
                dict.set_item("end_col", d.primary.end_col)?;
                dict.set_item("file", path)?;
                out.push(dict.unbind());
            }
        }
    }
    Ok(out)
}

#[pymodule]
fn aiproof(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

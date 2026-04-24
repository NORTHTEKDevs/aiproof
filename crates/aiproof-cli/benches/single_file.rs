// Criterion benchmark over representative sample prompts.
// Run: cargo bench -p aiproof-cli
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::path::PathBuf;

fn bench_parse_and_check(c: &mut Criterion) {
    let prompts = sample_prompts();
    c.bench_function("aiproof_parse_check_10_prompts", |b| {
        b.iter_batched(
            || prompts.clone(),
            |ps| {
                let rules = aiproof_rules::all_rules();
                let ctx = aiproof_core::rule::Ctx {
                    target_models: &[],
                    max_tokens_budget: None,
                };
                for (path, src) in &ps {
                    let docs = aiproof_parse::parse_file(path, src).unwrap_or_default();
                    for d in &docs {
                        for rule in &rules {
                            let _ = rule.check(d, &ctx);
                        }
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn sample_prompts() -> Vec<(PathBuf, String)> {
    let ten: &[(&str, &str)] = &[
        ("short.md", "Answer concisely."),
        ("json_ask.md", "Return your answer in JSON."),
        (
            "cred.prompt.md",
            "key=sk-ant-api03-abcdefghijklmnopqrstuvwxyz01",
        ),
        (
            "interp.md",
            "---\nrole: system\n---\nAnswer this: {query}\n",
        ),
        ("tone.md", "Be concise and thorough."),
        ("todo.md", "TODO: write the real system prompt."),
        ("overloaded.md", &"You must do X. ".repeat(20)),
        ("reason.md", "Think step by step."),
        (
            "mustache.mustache",
            "Hello {{name}}, your order #{{order_id}}.",
        ),
        (
            "jinja.j2",
            "{% for item in items %}- {{ item.title }}{% endfor %}",
        ),
    ];
    ten.iter()
        .map(|(n, s)| (PathBuf::from(n), s.to_string()))
        .collect()
}

criterion_group!(benches, bench_parse_and_check);
criterion_main!(benches);

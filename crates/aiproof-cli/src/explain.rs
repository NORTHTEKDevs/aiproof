/// Print bundled rule documentation.
static EXPLANATIONS: &[(&str, &str)] = &[
    ("AIP001", include_str!("../rules_docs/AIP001.md")),
    ("AIP002", include_str!("../rules_docs/AIP002.md")),
    ("AIP003", include_str!("../rules_docs/AIP003.md")),
    ("AIP004", include_str!("../rules_docs/AIP004.md")),
    ("AIP005", include_str!("../rules_docs/AIP005.md")),
    ("AIP006", include_str!("../rules_docs/AIP006.md")),
    ("AIP007", include_str!("../rules_docs/AIP007.md")),
    ("AIP008", include_str!("../rules_docs/AIP008.md")),
    ("AIP009", include_str!("../rules_docs/AIP009.md")),
    ("AIP010", include_str!("../rules_docs/AIP010.md")),
    ("AIP011", include_str!("../rules_docs/AIP011.md")),
    ("AIP012", include_str!("../rules_docs/AIP012.md")),
    ("AIP013", include_str!("../rules_docs/AIP013.md")),
    ("AIP014", include_str!("../rules_docs/AIP014.md")),
    ("AIP015", include_str!("../rules_docs/AIP015.md")),
    ("AIP016", include_str!("../rules_docs/AIP016.md")),
    ("AIP017", include_str!("../rules_docs/AIP017.md")),
    ("AIP018", include_str!("../rules_docs/AIP018.md")),
    ("AIP019", include_str!("../rules_docs/AIP019.md")),
    ("AIP020", include_str!("../rules_docs/AIP020.md")),
];

pub fn run_explain(code: &str) -> i32 {
    let upper = code.to_uppercase();
    match EXPLANATIONS.iter().find(|(c, _)| *c == upper) {
        Some((_, content)) => {
            print!("{content}");
            if !content.ends_with('\n') {
                println!();
            }
            0
        }
        None => {
            eprintln!("aiproof: no rule with code {upper}");
            eprintln!("(known codes: AIP001..AIP020)");
            2
        }
    }
}

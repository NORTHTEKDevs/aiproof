#![allow(clippy::collapsible_if, clippy::collapsible_else_if)]

use aiproof_core::document::{Document, Kind, PromptText, Role};
use aiproof_core::span::Span;
use std::path::Path;
use tree_sitter::{Node, Parser};

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::language())?;

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut docs = Vec::new();
    walk(tree.root_node(), source, path, &mut docs);
    Ok(docs)
}

fn walk<'a>(node: Node<'a>, source: &str, path: &Path, docs: &mut Vec<Document>) {
    if node.kind() == "call" {
        handle_call(node, source, path, docs);
    }

    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            walk(child, source, path, docs);
        }
    }
}

fn handle_call(call: Node, source: &str, path: &Path, docs: &mut Vec<Document>) {
    let Some(func) = call.child_by_field_name("function") else {
        return;
    };

    let dotted = dotted_tail(func, source);
    let args = call.child_by_field_name("arguments");

    match dotted.as_str() {
        s if s.ends_with("messages.create") => {
            extract_system_kwarg(call, args, source, path, docs, "python-anthropic");
            extract_messages_kwarg(call, args, source, path, docs);
            if let Some(temp) = extract_temperature_kwarg(args, source) {
                attach_temperature_to_last_n_docs(docs, temp, 2);
            }
        }
        s if s.ends_with("completions.create") => {
            extract_messages_kwarg(call, args, source, path, docs);
            if let Some(temp) = extract_temperature_kwarg(args, source) {
                attach_temperature_to_last_n_docs(docs, temp, 999);
            }
        }
        "PromptTemplate" => {
            extract_template_kwarg(call, args, source, path, docs);
        }
        "PromptTemplate.from_template" => {
            extract_first_positional_string(call, args, source, path, docs, Role::Unknown);
        }
        "ChatPromptTemplate.from_messages" => {
            extract_from_messages_list(call, args, source, path, docs);
        }
        "Agent" => {
            extract_system_kwarg(call, args, source, path, docs, "python-agent");
        }
        _ => {}
    }
}

/// Returns the dotted tail of an attribute expression, e.g. "messages.create".
fn dotted_tail(node: Node, source: &str) -> String {
    let mut parts = Vec::new();
    let mut current = node;

    loop {
        if current.kind() == "attribute" {
            if let Some(attr) = current.child_by_field_name("attribute") {
                if let Ok(name) = node_text(&attr, source) {
                    parts.push(name);
                }
            }
            if let Some(obj) = current.child_by_field_name("object") {
                current = obj;
                continue;
            }
        } else if current.kind() == "identifier" {
            if let Ok(name) = node_text(&current, source) {
                parts.push(name);
            }
        }
        break;
    }

    parts.reverse();
    parts.join(".")
}

fn node_text(node: &Node, source: &str) -> Result<String, ()> {
    let start = node.start_byte();
    let end = node.end_byte();
    if start < end && end <= source.len() {
        Ok(source[start..end].to_string())
    } else {
        Err(())
    }
}

fn extract_system_kwarg(
    call: Node,
    args: Option<Node>,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
    _origin: &str,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "keyword_argument" {
                if let Some(name) = child.child_by_field_name("name") {
                    if let Ok(name_text) = node_text(&name, source) {
                        if name_text == "system" {
                            if let Some(value) = child.child_by_field_name("value") {
                                if let Some((text, span)) = resolve_string_literal(value, source) {
                                    docs.push(Document {
                                        path: path.to_path_buf(),
                                        role: Role::System,
                                        source: source.to_string(),
                                        prompt: PromptText {
                                            text,
                                            origin_span: Some(span),
                                        },
                                        kind: Kind::ExtractedPython {
                                            call_site: Span::from_byte_range(
                                                source,
                                                call.start_byte()..call.end_byte(),
                                            ),
                                            temperature: None,
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn extract_messages_kwarg(
    _call: Node,
    args: Option<Node>,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "keyword_argument" {
                if let Some(name) = child.child_by_field_name("name") {
                    if let Ok(name_text) = node_text(&name, source) {
                        if name_text == "messages" {
                            if let Some(value) = child.child_by_field_name("value") {
                                extract_messages_from_list(value, source, path, docs);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn extract_temperature_kwarg(args: Option<Node>, source: &str) -> Option<f32> {
    let args = args?;

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "keyword_argument" {
                if let Some(name) = child.child_by_field_name("name") {
                    if let Ok(name_text) = node_text(&name, source) {
                        if name_text == "temperature" {
                            if let Some(value) = child.child_by_field_name("value") {
                                if let Ok(text) = node_text(&value, source) {
                                    if let Ok(temp) = text.parse::<f32>() {
                                        return Some(temp);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn attach_temperature_to_last_n_docs(docs: &mut [Document], temp: f32, n: usize) {
    let start = if docs.len() > n { docs.len() - n } else { 0 };
    for doc in &mut docs[start..] {
        if let Kind::ExtractedPython { temperature, .. } = &mut doc.kind {
            *temperature = Some(temp);
        }
    }
}

fn extract_messages_from_list(list: Node, source: &str, path: &Path, docs: &mut Vec<Document>) {
    if list.kind() != "list" {
        return;
    }

    for i in 0..list.named_child_count() {
        if let Some(child) = list.named_child(i) {
            if child.kind() == "dictionary" {
                extract_message_dict(child, source, path, docs);
            }
        }
    }
}

fn extract_message_dict(dict: Node, source: &str, path: &Path, docs: &mut Vec<Document>) {
    let mut role = None;
    let mut content = None;

    for i in 0..dict.named_child_count() {
        if let Some(child) = dict.named_child(i) {
            if child.kind() == "pair" {
                if let Some(key) = child.child_by_field_name("key") {
                    if let Some(val) = child.child_by_field_name("value") {
                        if let Ok(key_text) = node_text(&key, source) {
                            match key_text.trim_matches('\"').trim_matches('\'') {
                                "role" => {
                                    if let Ok(val_text) = node_text(&val, source) {
                                        role = Some(
                                            val_text
                                                .trim_matches('\"')
                                                .trim_matches('\'')
                                                .to_string(),
                                        );
                                    }
                                }
                                "content" => {
                                    content = resolve_string_literal(val, source);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    if let (Some(role_str), Some((text, origin_span))) = (role, content) {
        let role_enum = match role_str.as_str() {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::Unknown,
        };

        docs.push(Document {
            path: path.to_path_buf(),
            role: role_enum,
            source: source.to_string(),
            prompt: PromptText {
                text,
                origin_span: Some(origin_span),
            },
            kind: Kind::ExtractedPython {
                call_site: Span::from_byte_range(source, dict.start_byte()..dict.end_byte()),
                temperature: None,
            },
        });
    }
}

fn extract_template_kwarg(
    _call: Node,
    args: Option<Node>,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "keyword_argument" {
                if let Some(name) = child.child_by_field_name("name") {
                    if let Ok(name_text) = node_text(&name, source) {
                        if name_text == "template" {
                            if let Some(value) = child.child_by_field_name("value") {
                                if let Some((text, span)) = resolve_string_literal(value, source) {
                                    docs.push(Document {
                                        path: path.to_path_buf(),
                                        role: Role::Unknown,
                                        source: source.to_string(),
                                        prompt: PromptText {
                                            text,
                                            origin_span: Some(span),
                                        },
                                        kind: Kind::ExtractedPython {
                                            call_site: Span::from_byte_range(
                                                source,
                                                child.start_byte()..child.end_byte(),
                                            ),
                                            temperature: None,
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn extract_first_positional_string(
    call: Node,
    args: Option<Node>,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
    role: Role,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            let is_string_arg = child.kind() == "string" || child.kind() == "argument";
            if is_string_arg {
                if let Some((text, span)) = resolve_string_literal(child, source) {
                    docs.push(Document {
                        path: path.to_path_buf(),
                        role,
                        source: source.to_string(),
                        prompt: PromptText {
                            text,
                            origin_span: Some(span),
                        },
                        kind: Kind::ExtractedPython {
                            call_site: Span::from_byte_range(
                                source,
                                call.start_byte()..call.end_byte(),
                            ),
                            temperature: None,
                        },
                    });
                    return; // Only first positional
                }
            }
        }
    }
}

fn extract_from_messages_list(
    _call: Node,
    args: Option<Node>,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "list" {
                for j in 0..child.named_child_count() {
                    if let Some(item) = child.named_child(j) {
                        extract_from_messages_tuple(item, source, path, docs);
                    }
                }
            }
        }
    }
}

fn extract_from_messages_tuple(tuple: Node, source: &str, path: &Path, docs: &mut Vec<Document>) {
    if tuple.kind() != "tuple" {
        return;
    }

    let mut role = None;
    let mut content = None;

    for i in 0..tuple.named_child_count() {
        if let Some(child) = tuple.named_child(i) {
            match i {
                0 => {
                    if let Ok(text) = node_text(&child, source) {
                        role = Some(text.trim_matches('\"').trim_matches('\'').to_string());
                    }
                }
                1 => {
                    content = resolve_string_literal(child, source);
                }
                _ => {}
            }
        }
    }

    if let (Some(role_str), Some((text, origin_span))) = (role, content) {
        let role_enum = match role_str.as_str() {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::Unknown,
        };

        docs.push(Document {
            path: path.to_path_buf(),
            role: role_enum,
            source: source.to_string(),
            prompt: PromptText {
                text,
                origin_span: Some(origin_span),
            },
            kind: Kind::ExtractedPython {
                call_site: Span::from_byte_range(source, tuple.start_byte()..tuple.end_byte()),
                temperature: None,
            },
        });
    }
}

/// Resolve a string literal node to (text, origin_span).
/// Handles: plain strings, f-strings (with placeholder substitution), raw strings.
/// Returns None for dynamic expressions, names, etc.
fn resolve_string_literal(node: Node, source: &str) -> Option<(String, Span)> {
    if node.kind() != "string" {
        return None;
    }

    let start = node.start_byte();
    let end = node.end_byte();
    let span = Span::from_byte_range(source, start..end);

    let raw_text = &source[start..end];

    if raw_text.starts_with("f\"")
        || raw_text.starts_with("f'")
        || raw_text.starts_with("F\"")
        || raw_text.starts_with("F'")
        || raw_text.starts_with("rf\"")
        || raw_text.starts_with("fr\"")
        || raw_text.starts_with("rf'")
        || raw_text.starts_with("fr'")
    {
        let text = reconstruct_fstring(node, source);
        return Some((text, span));
    }

    if raw_text.starts_with("r\"")
        || raw_text.starts_with("r'")
        || raw_text.starts_with("R\"")
        || raw_text.starts_with("R'")
    {
        let quote_char = if raw_text.contains("\"\"\"") || raw_text.contains("'''") {
            &raw_text[2..5]
        } else {
            &raw_text[2..3]
        };
        let inner = extract_string_inner(raw_text, quote_char);
        return Some((inner, span));
    }

    let quote_char = if raw_text.starts_with("\"\"\"") || raw_text.starts_with("'''") {
        &raw_text[..3]
    } else {
        &raw_text[..1]
    };

    let inner = extract_string_inner(raw_text, quote_char);
    let unescaped = unescape_string(&inner);
    Some((unescaped, span))
}

fn extract_string_inner(raw: &str, quote: &str) -> String {
    if let Some(stripped) = raw
        .strip_prefix("rf")
        .or_else(|| raw.strip_prefix("fr"))
        .or_else(|| raw.strip_prefix("r"))
        .or_else(|| raw.strip_prefix("f"))
        .or_else(|| raw.strip_prefix("R"))
        .or_else(|| raw.strip_prefix("F"))
    {
        let stripped = stripped.strip_prefix(quote).unwrap_or(stripped);
        stripped.strip_suffix(quote).unwrap_or(stripped).to_string()
    } else {
        let stripped = raw.strip_prefix(quote).unwrap_or(raw);
        stripped.strip_suffix(quote).unwrap_or(stripped).to_string()
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some(&'n') => {
                    chars.next();
                    result.push('\n');
                }
                Some(&'t') => {
                    chars.next();
                    result.push('\t');
                }
                Some(&'r') => {
                    chars.next();
                    result.push('\r');
                }
                Some(&'\\') => {
                    chars.next();
                    result.push('\\');
                }
                Some(&'"') => {
                    chars.next();
                    result.push('"');
                }
                Some(&'\'') => {
                    chars.next();
                    result.push('\'');
                }
                _ => result.push(ch),
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn reconstruct_fstring(node: Node, source: &str) -> String {
    let mut result = String::new();
    let mut placeholder_index = 0;

    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            match child.kind() {
                "string_content" => {
                    if let Ok(text) = node_text(&child, source) {
                        let unescaped = unescape_string(&text);
                        result.push_str(&unescaped);
                    }
                }
                "interpolation" => {
                    result.push_str(&format!("{{{}}}", placeholder_index));
                    placeholder_index += 1;
                }
                _ => {}
            }
        }
    }

    if result.is_empty() {
        let start = node.start_byte();
        let end = node.end_byte();
        if start < end && end <= source.len() {
            let raw = &source[start..end];
            let quote = if raw.contains("\"\"\"") || raw.contains("'''") {
                &raw[..3]
            } else {
                &raw[2..3]
            };
            extract_string_inner(raw, quote)
        } else {
            String::new()
        }
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first(src: &str) -> Document {
        parse(Path::new("t.py"), src).unwrap().remove(0)
    }

    #[test]
    fn anthropic_system_extracted() {
        let src = r#"
client.messages.create(
    model="claude-4.7-opus",
    system="You are a helpful assistant.",
    messages=[{"role": "user", "content": "Hello"}],
)
"#;
        let d = first(src);
        assert_eq!(d.prompt.text, "You are a helpful assistant.");
        assert_eq!(d.role, Role::System);
    }

    #[test]
    fn openai_messages_extracted() {
        let src = r#"
openai.chat.completions.create(
    messages=[
        {"role": "system", "content": "Act as a tutor."},
        {"role": "user", "content": "Teach me fractions."},
    ],
)
"#;
        let docs = parse(Path::new("t.py"), src).unwrap();
        assert_eq!(docs.len(), 2);
        let sys = docs.iter().find(|d| d.role == Role::System).unwrap();
        assert_eq!(sys.prompt.text, "Act as a tutor.");
        let user = docs.iter().find(|d| d.role == Role::User).unwrap();
        assert_eq!(user.prompt.text, "Teach me fractions.");
    }

    #[test]
    fn prompttemplate_from_template() {
        let src = r#"PromptTemplate.from_template("Answer this: {q}")"#;
        let docs = parse(Path::new("t.py"), src).unwrap();
        assert!(
            !docs.is_empty(),
            "Expected at least one document, got {}",
            docs.len()
        );
        let d = &docs[0];
        assert_eq!(d.prompt.text, "Answer this: {q}");
    }

    #[test]
    fn chatprompttemplate_from_messages() {
        let src = r#"
ChatPromptTemplate.from_messages([
    ("system", "You are helpful."),
    ("user", "Q: {q}"),
])
"#;
        let docs = parse(Path::new("t.py"), src).unwrap();
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn fstring_becomes_positional_placeholder() {
        let src = r#"
client.messages.create(
    system=f"You are {name}. Tone: {tone}.",
    messages=[],
)
"#;
        let d = first(src);
        assert_eq!(d.prompt.text, "You are {0}. Tone: {1}.");
    }

    #[test]
    fn dynamic_expression_skipped() {
        let src = r#"
client.messages.create(
    system=SOMETHING_DYNAMIC,
    messages=[],
)
"#;
        let docs = parse(Path::new("t.py"), src).unwrap();
        assert!(docs.is_empty());
    }
}

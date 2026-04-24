#![allow(clippy::collapsible_if, clippy::collapsible_else_if)]

use aiproof_core::document::{Document, Kind, PromptText, Role};
use aiproof_core::span::Span;
use std::path::Path;
use tree_sitter::{Node, Parser};

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    let mut parser = Parser::new();

    let lang = if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s == "tsx")
        .unwrap_or(false)
    {
        tree_sitter_typescript::language_tsx()
    } else {
        tree_sitter_typescript::language_typescript()
    };

    parser.set_language(&lang)?;

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut docs = Vec::new();
    walk(tree.root_node(), source, path, &mut docs);
    Ok(docs)
}

fn walk<'a>(node: Node<'a>, source: &str, path: &Path, docs: &mut Vec<Document>) {
    if node.kind() == "call_expression" {
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
            extract_object_property(call, args, "system", source, path, docs, Role::System);
            extract_messages_property(call, args, source, path, docs);
            if let Some(temp) = extract_number_property(args, "temperature", source) {
                attach_temperature_to_last_n_docs(docs, temp, 2);
            }
        }
        s if s.ends_with("completions.create") => {
            extract_messages_property(call, args, source, path, docs);
            if let Some(temp) = extract_number_property(args, "temperature", source) {
                attach_temperature_to_last_n_docs(docs, temp, 999);
            }
        }
        "PromptTemplate" => {
            extract_object_property(call, args, "template", source, path, docs, Role::Unknown);
        }
        "PromptTemplate.fromTemplate" => {
            extract_first_positional_string(call, args, source, path, docs, Role::Unknown);
        }
        "ChatPromptTemplate.fromMessages" => {
            extract_from_messages_array(call, args, source, path, docs);
        }
        _ => {}
    }
}

/// Returns the dotted tail of a member expression, e.g. "messages.create".
fn dotted_tail(node: Node, source: &str) -> String {
    let mut parts = Vec::new();
    let mut current = node;

    loop {
        if current.kind() == "member_expression" {
            if let Some(prop) = current.child_by_field_name("property") {
                if let Ok(name) = node_text(&prop, source) {
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

fn extract_object_property(
    call: Node,
    args: Option<Node>,
    prop_name: &str,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
    role: Role,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "object" {
                if let Some(pair) = find_object_pair(child, prop_name, source) {
                    if let Some(value) = pair.child_by_field_name("value") {
                        if let Some((text, span)) = resolve_string_literal(value, source) {
                            docs.push(Document {
                                path: path.to_path_buf(),
                                role,
                                source: source.to_string(),
                                prompt: PromptText {
                                    text,
                                    origin_span: Some(span),
                                },
                                kind: Kind::ExtractedTypeScript {
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

fn extract_number_property(args: Option<Node>, prop_name: &str, source: &str) -> Option<f32> {
    let args = args?;

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "object" {
                if let Some(pair) = find_object_pair(child, prop_name, source) {
                    if let Some(value) = pair.child_by_field_name("value") {
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
    None
}

fn find_object_pair<'a>(obj: Node<'a>, prop_name: &str, source: &str) -> Option<Node<'a>> {
    for i in 0..obj.named_child_count() {
        if let Some(child) = obj.named_child(i) {
            if child.kind() == "pair" {
                if let Some(key) = child.child_by_field_name("key") {
                    if let Ok(key_text) = node_text(&key, source) {
                        let clean_key = key_text.trim_matches(|c| c == '"' || c == '\'');
                        if clean_key == prop_name {
                            return Some(child);
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_messages_property(
    _call: Node,
    args: Option<Node>,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "object" {
                if let Some(pair) = find_object_pair(child, "messages", source) {
                    if let Some(value) = pair.child_by_field_name("value") {
                        extract_messages_from_array(value, source, path, docs);
                    }
                }
            }
        }
    }
}

fn extract_messages_from_array(arr: Node, source: &str, path: &Path, docs: &mut Vec<Document>) {
    if arr.kind() != "array" {
        return;
    }

    for i in 0..arr.named_child_count() {
        if let Some(child) = arr.named_child(i) {
            if child.kind() == "object" {
                extract_message_object(child, source, path, docs);
            }
        }
    }
}

fn extract_message_object(obj: Node, source: &str, path: &Path, docs: &mut Vec<Document>) {
    let mut role = None;
    let mut content = None;

    for i in 0..obj.named_child_count() {
        if let Some(child) = obj.named_child(i) {
            if child.kind() == "pair" {
                if let Some(key) = child.child_by_field_name("key") {
                    if let Some(val) = child.child_by_field_name("value") {
                        if let Ok(key_text) = node_text(&key, source) {
                            let clean_key = key_text.trim_matches(|c| c == '"' || c == '\'');
                            match clean_key {
                                "role" => {
                                    if let Ok(val_text) = node_text(&val, source) {
                                        role = Some(
                                            val_text
                                                .trim_matches(|c| c == '"' || c == '\'')
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
            kind: Kind::ExtractedTypeScript {
                call_site: Span::from_byte_range(source, obj.start_byte()..obj.end_byte()),
                temperature: None,
            },
        });
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
            let is_string_arg = child.kind() == "string"
                || child.kind() == "template_string"
                || child.kind() == "template_literal";
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
                        kind: Kind::ExtractedTypeScript {
                            call_site: Span::from_byte_range(
                                source,
                                call.start_byte()..call.end_byte(),
                            ),
                            temperature: None,
                        },
                    });
                    return;
                }
            }
        }
    }
}

fn extract_from_messages_array(
    _call: Node,
    args: Option<Node>,
    source: &str,
    path: &Path,
    docs: &mut Vec<Document>,
) {
    let Some(args) = args else { return };

    for i in 0..args.named_child_count() {
        if let Some(child) = args.named_child(i) {
            if child.kind() == "array" {
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
    if tuple.kind() != "array" {
        return;
    }

    let mut role = None;
    let mut content = None;

    for i in 0..tuple.named_child_count() {
        if let Some(child) = tuple.named_child(i) {
            match i {
                0 => {
                    if let Ok(text) = node_text(&child, source) {
                        role = Some(text.trim_matches(|c| c == '"' || c == '\'').to_string());
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
            kind: Kind::ExtractedTypeScript {
                call_site: Span::from_byte_range(source, tuple.start_byte()..tuple.end_byte()),
                temperature: None,
            },
        });
    }
}

fn attach_temperature_to_last_n_docs(docs: &mut [Document], temp: f32, n: usize) {
    let start = if docs.len() > n { docs.len() - n } else { 0 };
    for doc in &mut docs[start..] {
        if let Kind::ExtractedTypeScript { temperature, .. } = &mut doc.kind {
            *temperature = Some(temp);
        }
    }
}

/// Resolve a string literal node to (text, origin_span).
/// Handles: plain strings, template literals (with placeholder substitution).
/// Returns None for dynamic expressions, identifiers, etc.
fn resolve_string_literal(node: Node, source: &str) -> Option<(String, Span)> {
    let kind = node.kind();

    let start = node.start_byte();
    let end = node.end_byte();
    let span = Span::from_byte_range(source, start..end);

    let raw_text = &source[start..end];

    if kind == "string" {
        let quote_char = if raw_text.starts_with('"') {
            '"'
        } else if raw_text.starts_with('\'') {
            '\''
        } else {
            return None;
        };

        let inner = extract_string_inner(raw_text, quote_char);
        let unescaped = unescape_string(&inner);
        return Some((unescaped, span));
    }

    if kind == "template_string" || kind == "template_literal" {
        let text = reconstruct_template_literal(node, source);
        return Some((text, span));
    }

    None
}

fn extract_string_inner(raw: &str, quote: char) -> String {
    let mut result = String::new();
    let mut chars = raw.chars();

    chars.next();

    for ch in chars {
        if ch == quote {
            break;
        }
        result.push(ch);
    }

    result
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

fn reconstruct_template_literal(node: Node, source: &str) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    if start >= end || end > source.len() {
        return String::new();
    }

    let raw = &source[start..end];
    if !raw.starts_with('`') || !raw.ends_with('`') {
        return String::new();
    }

    let mut result = String::new();
    let mut placeholder_index = 0;
    let inner = &raw[1..raw.len() - 1];
    let mut chars = inner.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            if chars.peek() == Some(&'{') {
                chars.next();
                let mut depth = 1;
                while depth > 0 && chars.peek().is_some() {
                    if chars.peek() == Some(&'{') {
                        depth += 1;
                    } else if chars.peek() == Some(&'}') {
                        depth -= 1;
                    }
                    chars.next();
                }
                result.push_str(&format!("{{{}}}", placeholder_index));
                placeholder_index += 1;
            } else {
                result.push(ch);
            }
        } else if ch == '\\' {
            if let Some(&next) = chars.peek() {
                chars.next();
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    _ => {
                        result.push(ch);
                        result.push(next);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ts(src: &str) -> Vec<Document> {
        parse(Path::new("t.ts"), src).unwrap()
    }

    fn first(src: &str) -> Document {
        parse_ts(src).remove(0)
    }

    #[test]
    fn anthropic_system_extracted() {
        let src = r#"
await client.messages.create({
  model: "claude-4.7-opus",
  system: "You are a helpful assistant.",
  messages: [{ role: "user", content: query }],
});
"#;
        let d = first(src);
        assert_eq!(d.prompt.text, "You are a helpful assistant.");
        assert_eq!(d.role, Role::System);
    }

    #[test]
    fn openai_messages_extracted() {
        let src = r#"
await openai.chat.completions.create({
  messages: [
    { role: "system", content: "Act as a tutor." },
    { role: "user", content: "Teach me fractions." },
  ],
});
"#;
        let docs = parse_ts(src);
        assert_eq!(docs.len(), 2);
        let sys = docs.iter().find(|d| d.role == Role::System).unwrap();
        assert_eq!(sys.prompt.text, "Act as a tutor.");
        let user = docs.iter().find(|d| d.role == Role::User).unwrap();
        assert_eq!(user.prompt.text, "Teach me fractions.");
    }

    #[test]
    fn prompttemplate_fromtemplate() {
        let src = r#"PromptTemplate.fromTemplate("Answer this: {q}")"#;
        let docs = parse_ts(src);
        assert!(!docs.is_empty());
        let d = &docs[0];
        assert_eq!(d.prompt.text, "Answer this: {q}");
    }

    #[test]
    fn chatprompttemplate_frommessages() {
        let src = r#"
ChatPromptTemplate.fromMessages([
  ["system", "You are helpful."],
  ["user", "Q: {q}"],
]);
"#;
        let docs = parse_ts(src);
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn template_literal_becomes_positional_placeholder() {
        let src = r#"
await client.messages.create({
  system: `You are ${name}. Tone: ${tone}.`,
  messages: [],
});
"#;
        let d = first(src);
        assert_eq!(d.prompt.text, "You are {0}. Tone: {1}.");
    }

    #[test]
    fn dynamic_expression_skipped() {
        let src = r#"
await client.messages.create({
  system: SOMETHING_DYNAMIC,
  messages: [],
});
"#;
        let docs = parse_ts(src);
        assert!(docs.is_empty());
    }
}

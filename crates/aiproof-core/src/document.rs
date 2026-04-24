use std::path::PathBuf;

use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub role: Role,
    pub source: String,
    pub prompt: PromptText,
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Unknown,
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct PromptText {
    pub text: String,
    pub origin_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum Kind {
    PlainText,
    Markdown,
    Jinja {
        variables: Vec<String>,
    },
    Mustache {
        variables: Vec<String>,
    },
    YamlConfig,
    JsonSchema,
    ExtractedPython {
        call_site: Span,
        #[allow(unused)]
        temperature: Option<f32>,
    },
    ExtractedTypeScript {
        call_site: Span,
        #[allow(unused)]
        temperature: Option<f32>,
    },
}

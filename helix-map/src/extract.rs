use crate::model::{Span, Symbol, SymbolKind, Visibility};
use anyhow::{Result, anyhow};
use std::path::Path;
use tracing::warn;
use tree_sitter::{Node, Parser};

pub trait SymbolExtractor {
    fn language(&self) -> crate::model::Language;
    fn extract(&self, path: &Path, source: &str) -> Result<Vec<Symbol>>;
}

#[derive(Debug, Default, Clone)]
pub struct RustExtractor;

impl RustExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl SymbolExtractor for RustExtractor {
    fn language(&self) -> crate::model::Language {
        crate::model::Language::Rust
    }

    fn extract(&self, path: &Path, source: &str) -> Result<Vec<Symbol>> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|_| anyhow!("failed to load Rust grammar"))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow!("failed to parse {}", path.display()))?;
        if tree.root_node().has_error() {
            warn!("parse errors detected in {}", path.display());
        }

        let mut symbols = Vec::new();
        collect_symbols(tree.root_node(), source, &mut symbols);
        Ok(symbols)
    }
}

fn collect_symbols(node: Node, source: &str, symbols: &mut Vec<Symbol>) {
    if let Some(symbol) = symbol_from_node(node, source) {
        symbols.push(symbol);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_symbols(child, source, symbols);
    }
}

fn symbol_from_node(node: Node, source: &str) -> Option<Symbol> {
    if is_within_block(node) {
        return None;
    }

    let kind = match node.kind() {
        "function_item" | "function_signature_item" => SymbolKind::Function,
        "struct_item" => SymbolKind::Struct,
        "enum_item" => SymbolKind::Enum,
        "trait_item" => SymbolKind::Trait,
        "type_item" => SymbolKind::TypeAlias,
        "const_item" => SymbolKind::Const,
        "mod_item" => SymbolKind::Module,
        _ => return None,
    };

    let name = node
        .child_by_field_name("name")
        .and_then(|child| child.utf8_text(source.as_bytes()).ok())
        .map(str::to_string)?;

    let (visibility, visibility_text) = visibility_from_node(node, source);
    let signature = match kind {
        SymbolKind::Function => function_signature(node, source, &name, visibility_text),
        SymbolKind::Struct => type_signature("struct", node, source, &name, visibility_text),
        SymbolKind::Enum => type_signature("enum", node, source, &name, visibility_text),
        SymbolKind::Trait => type_signature("trait", node, source, &name, visibility_text),
        SymbolKind::TypeAlias => type_signature("type", node, source, &name, visibility_text),
        SymbolKind::Const => type_signature("const", node, source, &name, visibility_text),
        SymbolKind::Module => type_signature("mod", node, source, &name, visibility_text),
        SymbolKind::Method => function_signature(node, source, &name, visibility_text),
    };

    let kind = if kind == SymbolKind::Function && is_method(node) {
        SymbolKind::Method
    } else {
        kind
    };

    Some(Symbol {
        name,
        kind,
        visibility,
        signature,
        span: Span {
            start_line: node.start_position().row.saturating_add(1) as u32,
            end_line: node.end_position().row.saturating_add(1) as u32,
        },
    })
}

fn is_within_block(node: Node) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if parent.kind() == "block" {
            return true;
        }
        if matches!(
            parent.kind(),
            "impl_item" | "trait_item" | "mod_item" | "source_file"
        ) {
            return false;
        }
        current = parent;
    }
    false
}

fn is_method(node: Node) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if parent.kind() == "impl_item" || parent.kind() == "trait_item" {
            return true;
        }
        if parent.kind() == "source_file" || parent.kind() == "mod_item" {
            break;
        }
        current = parent;
    }
    false
}

fn visibility_from_node(node: Node, source: &str) -> (Visibility, Option<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier"
            && let Ok(text) = child.utf8_text(source.as_bytes())
        {
            let visibility = visibility_from_text(text);
            return (visibility, Some(normalize_whitespace(text)));
        }
    }
    (Visibility::Private, None)
}

fn visibility_from_text(text: &str) -> Visibility {
    if text.starts_with("pub(") {
        if text.contains("crate") {
            Visibility::Crate
        } else {
            Visibility::Restricted
        }
    } else if text.starts_with("pub") {
        Visibility::Public
    } else {
        Visibility::Private
    }
}

fn function_signature(node: Node, source: &str, name: &str, visibility: Option<String>) -> String {
    let mut signature = String::new();
    if let Some(vis) = visibility {
        signature.push_str(&vis);
        signature.push(' ');
    }

    let mut modifier_parts = Vec::new();
    collect_modifiers(node, source, &mut modifier_parts);
    if !modifier_parts.is_empty() {
        signature.push_str(&modifier_parts.join(" "));
        signature.push(' ');
    }

    signature.push_str("fn ");
    signature.push_str(name);

    if let Some(type_params) = text_for_field(node, source, "type_parameters") {
        signature.push_str(&type_params);
    }

    if let Some(parameters) = text_for_field(node, source, "parameters") {
        signature.push_str(&parameters);
    } else {
        signature.push_str("()");
    }

    if let Some(return_type) = text_for_field(node, source, "return_type") {
        signature.push(' ');
        signature.push_str(&return_type);
    }

    if let Some(where_clause) = text_for_field(node, source, "where_clause") {
        signature.push(' ');
        signature.push_str(&where_clause);
    }

    normalize_whitespace(&signature)
}

fn type_signature(
    keyword: &str,
    node: Node,
    source: &str,
    name: &str,
    visibility: Option<String>,
) -> String {
    let mut signature = String::new();
    if let Some(vis) = visibility {
        signature.push_str(&vis);
        signature.push(' ');
    }
    signature.push_str(keyword);
    signature.push(' ');
    signature.push_str(name);

    if let Some(type_params) = text_for_field(node, source, "type_parameters") {
        signature.push_str(&type_params);
    }

    normalize_whitespace(&signature)
}

fn text_for_field(node: Node, source: &str, field: &str) -> Option<String> {
    node.child_by_field_name(field)
        .and_then(|child| child.utf8_text(source.as_bytes()).ok())
        .map(normalize_whitespace)
}

fn collect_modifiers(node: Node, source: &str, modifiers: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if matches!(kind, "async" | "const" | "unsafe" | "extern_modifier")
            && let Ok(text) = child.utf8_text(source.as_bytes())
        {
            modifiers.push(normalize_whitespace(text));
        }
    }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

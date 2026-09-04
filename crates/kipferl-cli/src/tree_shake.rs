use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeProfile {
    Core,
    Full,
}

impl RuntimeProfile {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Analysis {
    pub(crate) profile: RuntimeProfile,
    pub(crate) imports: BTreeSet<String>,
    pub(crate) reasons: Vec<String>,
}

impl Analysis {
    pub(crate) fn forced_full() -> Self {
        Self {
            profile: RuntimeProfile::Full,
            imports: BTreeSet::new(),
            reasons: vec!["disabled with --full-runtime".to_owned()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Name(String),
    Dot,
    Comma,
    Newline,
    Semicolon,
    LeftParenthesis,
    RightParenthesis,
    Other,
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The token cursor only advances after successful slice lookups; allocated slices are limited to isize::MAX, leaving room for the at-most-two-token lookahead"
)]
pub fn analyze(source: &str) -> Analysis {
    let tokens = tokenize(source);
    let mut imports = BTreeSet::new();
    let mut reasons = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        match tokens.get(index) {
            Some(Token::Name(name)) if name == "from" => {
                index = parse_from_import(&tokens, index + 1, &mut imports, &mut reasons);
            }
            Some(Token::Name(name)) if name == "import" => {
                index = parse_import(&tokens, index + 1, &mut imports);
            }
            Some(Token::Name(name))
                if matches!(name.as_str(), "__import__" | "importlib" | "exec" | "eval") =>
            {
                reasons.push(format!("dynamic code or import via {name}"));
                index += 1;
            }
            _ => index += 1,
        }
    }

    for module in &imports {
        if let Some(capability) = full_runtime_capability(module) {
            reasons.push(format!("{module} requires the {capability} capability"));
        }
    }
    reasons.sort();
    reasons.dedup();

    Analysis {
        profile: if reasons.is_empty() {
            RuntimeProfile::Core
        } else {
            RuntimeProfile::Full
        },
        imports,
        reasons,
    }
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The token cursor only advances after successful slice lookups; allocated slices are limited to isize::MAX, leaving room for the at-most-two-token lookahead"
)]
fn parse_from_import(
    tokens: &[Token],
    mut index: usize,
    imports: &mut BTreeSet<String>,
    reasons: &mut Vec<String>,
) -> usize {
    if matches!(tokens.get(index), Some(Token::Dot)) {
        reasons.push("relative import cannot be resolved statically".to_owned());
        return skip_statement(tokens, index);
    }

    let Some((module, next)) = dotted_name(tokens, index) else {
        reasons.push("unrecognized from-import syntax".to_owned());
        return skip_statement(tokens, index);
    };
    index = next;
    while matches!(tokens.get(index), Some(Token::Newline)) {
        index += 1;
    }
    if !matches!(tokens.get(index), Some(Token::Name(name)) if name == "import") {
        reasons.push("unrecognized from-import syntax".to_owned());
        return skip_statement(tokens, index);
    }
    imports.insert(module);
    skip_statement(tokens, index + 1)
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The token cursor only advances after successful slice lookups; allocated slices are limited to isize::MAX, leaving room for the at-most-two-token lookahead"
)]
fn parse_import(tokens: &[Token], mut index: usize, imports: &mut BTreeSet<String>) -> usize {
    loop {
        while matches!(
            tokens.get(index),
            Some(Token::LeftParenthesis | Token::RightParenthesis | Token::Newline)
        ) {
            index += 1;
        }
        let Some((module, next)) = dotted_name(tokens, index) else {
            return skip_statement(tokens, index);
        };
        imports.insert(module);
        index = next;

        while let Some(token) = tokens.get(index) {
            match token {
                Token::Comma => {
                    index += 1;
                    break;
                }
                Token::Newline | Token::Semicolon => return index + 1,
                _ => index += 1,
            }
        }
        if index >= tokens.len() {
            return index;
        }
    }
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The token cursor only advances after successful slice lookups; allocated slices are limited to isize::MAX, leaving room for the at-most-two-token lookahead"
)]
fn dotted_name(tokens: &[Token], mut index: usize) -> Option<(String, usize)> {
    let Token::Name(first) = tokens.get(index)? else {
        return None;
    };
    let mut module = first.clone();
    index += 1;
    while matches!(tokens.get(index), Some(Token::Dot)) {
        let Some(Token::Name(component)) = tokens.get(index + 1) else {
            break;
        };
        module.push('.');
        module.push_str(component);
        index += 2;
    }
    Some((module, index))
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The token cursor only advances after successful slice lookups; allocated slices are limited to isize::MAX, leaving room for the at-most-two-token lookahead"
)]
fn skip_statement(tokens: &[Token], mut index: usize) -> usize {
    while !matches!(
        tokens.get(index),
        None | Some(Token::Newline | Token::Semicolon)
    ) {
        index += 1;
    }
    index.saturating_add(1).min(tokens.len())
}

fn full_runtime_capability(module: &str) -> Option<&'static str> {
    let root = module.split('.').next().unwrap_or(module);
    match root {
        "gzip" | "tarfile" | "zipfile" => Some("archive"),
        "hashlib" | "hmac" => Some("crypto"),
        "http" => Some("HTTP/TLS"),
        "importlib" => Some("dynamic import"),
        "input" => Some("interactive Ratatui"),
        "kdl" | "toml" | "tomllib" | "yaml" => Some("extended format"),
        "re" => Some("regex"),
        "sqlite3" => Some("SQLite"),
        "time" => Some("timezone"),
        _ => None,
    }
}

fn tokenize(source: &str) -> Vec<Token> {
    tokenize_spanned(source)
        .into_iter()
        .map(|(token, _, _)| token)
        .collect()
}

#[expect(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::string_slice,
    reason = "Every byte access is guarded by index < bytes.len(); lookahead uses get; cursor increments are at most three and source slices cover only scanned ASCII identifiers"
)]
fn tokenize_spanned(source: &str) -> Vec<(Token, usize, usize)> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let start_offset = index;
        match bytes[index] {
            b'#' => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            quote @ (b'\'' | b'"') => {
                let triple =
                    bytes.get(index + 1) == Some(&quote) && bytes.get(index + 2) == Some(&quote);
                index += if triple { 3 } else { 1 };
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if triple
                        && bytes.get(index) == Some(&quote)
                        && bytes.get(index + 1) == Some(&quote)
                        && bytes.get(index + 2) == Some(&quote)
                    {
                        index += 3;
                        break;
                    } else if !triple && bytes[index] == quote {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
            }
            b'\\' if bytes.get(index + 1) == Some(&b'\n') => index += 2,
            b'\n' => {
                tokens.push((Token::Newline, start_offset, index + 1));
                index += 1;
            }
            b'.' => {
                tokens.push((Token::Dot, start_offset, index + 1));
                index += 1;
            }
            b',' => {
                tokens.push((Token::Comma, start_offset, index + 1));
                index += 1;
            }
            b';' => {
                tokens.push((Token::Semicolon, start_offset, index + 1));
                index += 1;
            }
            b'(' => {
                tokens.push((Token::LeftParenthesis, start_offset, index + 1));
                index += 1;
            }
            b')' => {
                tokens.push((Token::RightParenthesis, start_offset, index + 1));
                index += 1;
            }
            byte if byte == b'_' || byte.is_ascii_alphabetic() => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && (bytes[index] == b'_' || bytes[index].is_ascii_alphanumeric())
                {
                    index += 1;
                }
                tokens.push((
                    Token::Name(source[start..index].to_owned()),
                    start_offset,
                    index,
                ));
            }
            byte if byte.is_ascii_whitespace() => index += 1,
            _ => {
                tokens.push((Token::Other, start_offset, index + 1));
                index += 1;
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    #[test]
    fn malformed_and_unicode_sources_keep_import_spans_in_bounds() {
        for source in [
            "é = '💥'\nimport local\n",
            "from pkg import (\n name,\n",
            "import pkg # é\n",
            "'''unterminated 💥",
            "import pkg; print('é')",
            "from . import member",
            "import pkg\\\n, other",
        ] {
            for statement in super::import_statements(source) {
                assert!(
                    source.get(statement.start..statement.end).is_some(),
                    "{source:?}"
                );
            }
            let _ = super::analyze(source);
        }
    }

    use super::{RuntimeProfile, analyze};

    #[test]
    fn keeps_static_core_imports_in_the_pruned_runtime() {
        let analysis =
            analyze("import os, json as json_module\nfrom xml.etree.ElementTree import parse\n");
        assert_eq!(analysis.profile, RuntimeProfile::Core);
        assert_eq!(
            analysis.imports.into_iter().collect::<Vec<_>>(),
            ["json", "os", "xml.etree.ElementTree"]
        );
        assert!(analysis.reasons.is_empty());
    }

    #[test]
    fn selects_full_for_every_optional_capability() {
        for module in [
            "gzip",
            "hashlib",
            "hmac",
            "http.client",
            "input",
            "kdl",
            "re",
            "sqlite3",
            "tarfile",
            "time",
            "toml",
            "tomllib",
            "yaml",
            "zipfile",
        ] {
            let analysis = analyze(&format!("import {module}\n"));
            assert_eq!(analysis.profile, RuntimeProfile::Full, "{module}");
            assert!(!analysis.reasons.is_empty(), "{module}");
        }
    }

    #[test]
    fn understands_multiline_from_and_multiple_imports() {
        let analysis = analyze(
            "from yaml import (\n  safe_load,\n  safe_dump,\n)\nimport os, sqlite3 as db\n",
        );
        assert_eq!(analysis.profile, RuntimeProfile::Full);
        assert!(analysis.imports.contains("yaml"));
        assert!(analysis.imports.contains("sqlite3"));
    }

    #[test]
    fn ignores_import_words_inside_comments_and_strings() {
        let analysis = analyze(
            "# import sqlite3\nmessage = '''import yaml'''\nprint(\"import time\")\nimport json\n",
        );
        assert_eq!(analysis.profile, RuntimeProfile::Core);
        assert_eq!(analysis.imports.into_iter().collect::<Vec<_>>(), ["json"]);
    }

    #[test]
    fn dynamic_and_relative_imports_fall_back_to_full() {
        for source in [
            "module = __import__('json')\n",
            "import importlib\n",
            "exec(source)\n",
            "from .helpers import value\n",
        ] {
            assert_eq!(analyze(source).profile, RuntimeProfile::Full, "{source}");
        }
    }
}

/// Source offsets let packaging insert import preparation without moving user lines.
#[derive(Debug)]
pub struct ImportStatement {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) modules: Vec<String>,
    pub(crate) members: Vec<String>,
    pub(crate) aliases: Vec<Option<String>>,
    pub(crate) is_from: bool,
}

#[expect(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "Each token index is guarded by its vector length; start/end delimit a scanned token interval and all increments are bounded by the allocated vector plus two lookahead tokens"
)]
pub fn import_statements(source: &str) -> Vec<ImportStatement> {
    let tokens = tokenize_spanned(source);
    let mut statements = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let is_from = matches!(&tokens[index].0, Token::Name(name) if name == "from");
        if !is_from && !matches!(&tokens[index].0, Token::Name(name) if name == "import") {
            index += 1;
            continue;
        }
        let start = tokens[index].1;
        let mut end = index + 1;
        let mut depth = 0_usize;
        while end < tokens.len() {
            match tokens[end].0 {
                Token::LeftParenthesis => depth += 1,
                Token::RightParenthesis => depth = depth.saturating_sub(1),
                Token::Newline | Token::Semicolon if depth == 0 => break,
                _ => {}
            }
            end += 1;
        }
        let statement_tokens: Vec<Token> = tokens[index + 1..end]
            .iter()
            .map(|(token, _, _)| token.clone())
            .collect();
        let mut modules = Vec::new();
        let mut members = Vec::new();
        let mut aliases = Vec::new();
        let mut cursor = 0;
        if is_from {
            let mut name = String::new();
            while matches!(statement_tokens.get(cursor), Some(Token::Dot)) {
                name.push('.');
                cursor += 1;
            }
            if let Some((tail, next)) = dotted_name(&statement_tokens, cursor) {
                // A bare relative import is followed immediately by the keyword.
                if tail != "import" {
                    name.push_str(&tail);
                    cursor = next;
                }
            }
            if matches!(statement_tokens.get(cursor), Some(Token::Name(token)) if token == "import")
            {
                modules.push(name);
                cursor += 1;
                let mut expect_name = true;
                while cursor < statement_tokens.len() {
                    match &statement_tokens[cursor] {
                        Token::Name(name) if expect_name => {
                            members.push(name.clone());
                            expect_name = false;
                        }
                        Token::Comma => expect_name = true,
                        _ => {}
                    }
                    cursor += 1;
                }
            }
        } else {
            while cursor < statement_tokens.len() {
                if let Some((name, next)) = dotted_name(&statement_tokens, cursor) {
                    modules.push(name);
                    cursor = next;
                    aliases.push(
                        match (
                            statement_tokens.get(cursor),
                            statement_tokens.get(cursor + 1),
                        ) {
                            (Some(Token::Name(keyword)), Some(Token::Name(alias)))
                                if keyword == "as" =>
                            {
                                Some(alias.clone())
                            }
                            _ => None,
                        },
                    );
                }
                while cursor < statement_tokens.len() && statement_tokens[cursor] != Token::Comma {
                    cursor += 1;
                }
                cursor += 1;
            }
        }
        if !modules.is_empty() {
            statements.push(ImportStatement {
                start,
                end: tokens.get(end).map_or(source.len(), |(_, start, _)| *start),
                modules,
                members,
                aliases,
                is_from,
            });
        }
        index = end;
    }
    statements
}

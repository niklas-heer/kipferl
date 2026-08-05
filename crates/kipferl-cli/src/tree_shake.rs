use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeProfile {
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
pub(crate) struct Analysis {
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

pub(crate) fn analyze(source: &str) -> Analysis {
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
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
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
                tokens.push(Token::Newline);
                index += 1;
            }
            b'.' => {
                tokens.push(Token::Dot);
                index += 1;
            }
            b',' => {
                tokens.push(Token::Comma);
                index += 1;
            }
            b';' => {
                tokens.push(Token::Semicolon);
                index += 1;
            }
            b'(' => {
                tokens.push(Token::LeftParenthesis);
                index += 1;
            }
            b')' => {
                tokens.push(Token::RightParenthesis);
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
                tokens.push(Token::Name(source[start..index].to_owned()));
            }
            byte if byte.is_ascii_whitespace() => index += 1,
            _ => {
                tokens.push(Token::Other);
                index += 1;
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
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

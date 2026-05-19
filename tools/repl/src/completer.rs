use std::collections::HashSet;

pub struct Completer {
    keywords: Vec<&'static str>,
    builtins: Vec<&'static str>,
}

impl Completer {
    pub fn new() -> Self {
        Completer {
            keywords: vec![
                "let", "var", "const", "fn", "return", "if", "else", "elif",
                "for", "while", "match", "in", "is", "not", "and", "or",
                "true", "false", "null", "type", "interface", "trait", "impl",
                "enum", "struct", "import", "export", "from", "as",
                "async", "await", "spawn", "channel", "send", "recv",
                "pub", "priv", "static", "const", "mut", "self", "super",
                "test", "assert", "ai", "extend", "where",
            ],
            builtins: vec![
                "print", "println", "len", "type", "typeof", "panic",
                "assert", "assert_eq", "assert_ne",
                "range", "zip", "map", "filter", "reduce",
                "str", "int", "float", "bool", "char",
                "open", "read", "write", "close",
                "json", "http", "time", "random",
            ],
        }
    }

    pub fn complete(&self, input: &str) -> Vec<String> {
        let input = input.trim();
        if input.is_empty() {
            return Vec::new();
        }

        let mut matches: Vec<String> = Vec::new();
        let lower = input.to_lowercase();

        // Check keywords
        for kw in &self.keywords {
            if kw.starts_with(&lower) {
                matches.push(kw.to_string());
            }
        }

        // Check builtins
        for b in &self.builtins {
            if b.starts_with(&lower) {
                matches.push(b.to_string());
            }
        }

        // Remove duplicates
        let mut seen = HashSet::new();
        matches.retain(|m| seen.insert(m.clone()));

        matches
    }
}

impl Default for Completer {
    fn default() -> Self { Self::new() }
}
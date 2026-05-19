use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InitializeParams {
    process_id: Option<i64>,
    root_uri: Option<String>,
    capabilities: ClientCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientCapabilities {
    text_document: Option<TextDocumentClientCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextDocumentClientCapabilities {
    synchronization: Option<SyncCapability>,
    completion: Option<CompletionCapability>,
    hover: Option<HoverCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncCapability {
    dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompletionCapability {
    dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HoverCapability {
    dynamic_registration: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextDocumentPositionParams {
    text_document: TextDocumentIdentifier,
    position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextDocumentIdentifier {
    uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    line: u64,
    character: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Range {
    start: Position,
    end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Location {
    uri: String,
    range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Hover {
    contents: MarkupContent,
    range: Option<Range>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarkupContent {
    kind: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompletionItem {
    label: String,
    kind: Option<i32>,
    detail: Option<String>,
    documentation: Option<String>,
    insert_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Diagnostic {
    range: Range,
    severity: Option<i32>,
    message: String,
}

#[derive(Debug, Clone)]
struct Document {
    text: String,
    version: i64,
}

struct LSPState {
    documents: HashMap<String, Document>,
    completion_items: Vec<CompletionItem>,
}

impl LSPState {
    fn new() -> Self {
        let mut completion_items = Vec::new();
        
        // Keywords
        for kw in &["let", "var", "const", "fn", "return", "if", "else", "elif", "for", "while", 
                    "match", "in", "is", "not", "and", "or", "true", "false", "null", "type", 
                    "interface", "trait", "impl", "enum", "struct", "import", "export", "async", 
                    "await", "spawn", "pub", "priv", "mut", "self", "test", "assert"] {
            completion_items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(14), // keyword
                detail: None,
                documentation: None,
                insert_text: None,
            });
        }

        // Types
        for ty in &["Int", "Float", "Bool", "Str", "Char", "Array", "Map", "Option", "Result"] {
            completion_items.push(CompletionItem {
                label: ty.to_string(),
                kind: Some(7), // type
                detail: Some(format!("type {}", ty)),
                documentation: None,
                insert_text: None,
            });
        }

        LSPState {
            documents: HashMap::new(),
            completion_items,
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut input = String::new();
    let mut state = LSPState::new();

    println!("Flint LSP server starting...");

    loop {
        input.clear();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let msg: serde_json::Value = match serde_json::from_str(&input) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Some(id) = msg.get("id") {
                    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
                    let params = msg.get("params");

                    let response = handle_request(&mut state, method, params);
                    
                    let response_json = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": response
                    });

                    let _ = stdout.write_all(format!("{}\n", response_json).as_bytes());
                    let _ = stdout.flush();
                }
            }
            Err(_) => break,
        }
    }
}

fn handle_request(state: &mut LSPState, method: &str, params: Option<&serde_json::Value>) -> serde_json::Value {
    match method {
        "initialize" => {
            serde_json::json!({
                "capabilities": {
                    "text_document_sync": 1,
                    "completion_provider": { "trigger_characters": [".", ":", "("] },
                    "hover_provider": true,
                    "definition_provider": true,
                    "references_provider": true,
                },
                "server_info": {
                    "name": "flint-lsp",
                    "version": "0.1.0"
                }
            })
        }
        "text_document/did_open" => {
            if let Some(params) = params {
                let uri = params.get("text_document").and_then(|t| t.get("uri")).and_then(|u| u.as_str()).unwrap_or("");
                let text = params.get("text_document").and_then(|t| t.get("text")).and_then(|t| t.as_str()).unwrap_or("");
                let version = params.get("text_document").and_then(|t| t.get("version")).and_then(|v| v.as_i64()).unwrap_or(0);
                
                state.documents.insert(uri.to_string(), Document { text: text.to_string(), version });
            }
            serde_json::Value::Null
        }
        "text_document/did_change" => {
            if let Some(params) = params {
                let uri = params.get("text_document").and_then(|t| t.get("uri")).and_then(|u| u.as_str()).unwrap_or("");
                if let Some(doc) = state.documents.get_mut(uri) {
                    if let Some(content_changes) = params.get("content_changes") {
                        if let Some(change) = content_changes.as_array().and_then(|a| a.first()) {
                            if let Some(text) = change.get("text").and_then(|t| t.as_str()) {
                                doc.text = text.to_string();
                            }
                        }
                    }
                }
            }
            serde_json::Value::Null
        }
        "text_document/completion" => {
            let items: Vec<_> = state.completion_items.iter()
                .map(|c| serde_json::json!({
                    "label": c.label,
                    "kind": c.kind,
                    "detail": c.detail
                }))
                .collect();
            serde_json::json!(items)
        }
        "text_document/hover" => {
            if let Some(params) = params {
                let position = params.get("position").and_then(|p| p.get("line")).and_then(|l| l.as_u64()).unwrap_or(0);
                let character = params.get("position").and_then(|p| p.get("character")).and_then(|c| c.as_u64()).unwrap_or(0);

                serde_json::json!({
                    "contents": {
                        "kind": "markdown",
                        "value": "**Flint**\n\nA programming language."
                    },
                    "range": {
                        "start": { "line": position, "character": character },
                        "end": { "line": position, "character": character + 10 }
                    }
                })
            } else {
                serde_json::Value::Null
            }
        }
        "text_document/definition" => {
            serde_json::json!(null)
        }
        "text_document/publish_diagnostics" => {
            serde_json::Value::Null
        }
        _ => serde_json::Value::Null,
    }
}

fn parse_position(text: &str, line: u64, character: u64) -> Option<usize> {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = line as usize;
    
    if line_idx >= lines.len() {
        return None;
    }
    
    let line_text = lines[line_idx];
    let char_idx = character as usize;
    
    if char_idx > line_text.len() {
        return None;
    }
    
    Some(line_text[..char_idx].chars().count())
}
pub mod client;
pub mod prompt;
pub mod agent;
pub mod embeddings;
pub mod streaming;
pub mod tools;

pub use client::{AiClient, Model, ModelRegistry, CompletionRequest, CompletionResponse};
pub use prompt::{prompt, PromptBuilder, PromptTemplate};
pub use agent::{Agent, AgentBuilder, AgentState, Tool};
pub use embeddings::{Embedder, Embedding, EmbeddingRequest};
pub use streaming::StreamResponse;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

static GLOBAL_CLIENT: once_cell::sync::OnceCell<AiClient> = once_cell::sync::OnceCell::new();

pub fn init_client(config: AiConfig) {
    let client = AiClient::new(config);
    GLOBAL_CLIENT.set(client).ok();
}

pub fn global_client() -> Option<&'static AiClient> {
    GLOBAL_CLIENT.get()
}

pub fn default_model() -> String {
    "claude-3".to_string()
}

pub struct AiConfig {
    pub provider: AiProvider,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub timeout_ms: Option<u64>,
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            provider: AiProvider::OpenAI,
            api_key: None,
            base_url: None,
            default_model: "claude-3".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            timeout_ms: Some(60000),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Ollama,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct AiContext {
    pub history: Vec<PromptMessage>,
    pub tools: Vec<ToolDef>,
    pub system_prompt: Option<String>,
}

impl Default for AiContext {
    fn default() -> Self {
        AiContext {
            history: Vec::new(),
            tools: Vec::new(),
            system_prompt: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, ParamSpec>,
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

pub fn chat(messages: Vec<PromptMessage>) -> Result<String, AiError> {
    let client = global_client().ok_or(AiError::NotInitialized)?;
    client.chat(messages)
}

pub fn complete(prompt: &str) -> Result<String, AiError> {
    let client = global_client().ok_or(AiError::NotInitialized)?;
    client.complete(prompt)
}

pub async fn complete_async(prompt: &str) -> Result<String, AiError> {
    let client = global_client().ok_or(AiError::NotInitialized)?;
    client.complete_async(prompt).await
}

pub fn embed(text: &str) -> Result<Vec<f32>, AiError> {
    let client = global_client().ok_or(AiError::NotInitialized)?;
    client.embed(text)
}

#[derive(Debug)]
pub enum AiError {
    NotInitialized,
    ApiError(String),
    NetworkError(String),
    Timeout,
    InvalidResponse,
    RateLimited,
    AuthenticationFailed,
    ModelNotFound(String),
    InvalidConfig(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::NotInitialized => write!(f, "AI client not initialized"),
            AiError::ApiError(s) => write!(f, "API error: {}", s),
            AiError::NetworkError(s) => write!(f, "Network error: {}", s),
            AiError::Timeout => write!(f, "Request timeout"),
            AiError::InvalidResponse => write!(f, "Invalid response from API"),
            AiError::RateLimited => write!(f, "Rate limited"),
            AiError::AuthenticationFailed => write!(f, "Authentication failed"),
            AiError::ModelNotFound(s) => write!(f, "Model not found: {}", s),
            AiError::InvalidConfig(s) => write!(f, "Invalid config: {}", s),
        }
    }
}

impl std::error::Error for AiError {}

pub fn debug<T>(value: T, label: &str) -> String {
    format!("[DEBUG {}] {:?}", label, value)
}

pub fn explain<T: std::fmt::Debug>(value: T) -> String {
    format!("{:?}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = AiConfig::default();
        assert_eq!(config.default_model, "claude-3");
        assert_eq!(config.provider, AiProvider::OpenAI);
    }

    #[test]
    fn test_context_default() {
        let ctx = AiContext::default();
        assert!(ctx.history.is_empty());
        assert!(ctx.tools.is_empty());
        assert!(ctx.system_prompt.is_none());
    }

    #[test]
    fn test_message_roles() {
        let msg = PromptMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        assert_eq!(msg.role, MessageRole::User);
    }
}
use crate::{AiConfig, AiError, AiProvider, MessageRole, PromptMessage};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct AiClient {
    config: AiConfig,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub provider: AiProvider,
    pub supports_streaming: bool,
    pub max_tokens: usize,
    pub supports_functions: bool,
}

pub struct ModelRegistry {
    models: HashMap<String, Model>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        let mut models = HashMap::new();
        
        models.insert("gpt-4".to_string(), Model {
            name: "gpt-4".to_string(),
            provider: AiProvider::OpenAI,
            supports_streaming: true,
            max_tokens: 8192,
            supports_functions: true,
        });
        
        models.insert("gpt-3.5-turbo".to_string(), Model {
            name: "gpt-3.5-turbo".to_string(),
            provider: AiProvider::OpenAI,
            supports_streaming: true,
            max_tokens: 4096,
            supports_functions: true,
        });
        
        models.insert("claude-3".to_string(), Model {
            name: "claude-3".to_string(),
            provider: AiProvider::Anthropic,
            supports_streaming: true,
            max_tokens: 4096,
            supports_functions: true,
        });
        
        models.insert("llama3".to_string(), Model {
            name: "llama3".to_string(),
            provider: AiProvider::Ollama,
            supports_streaming: true,
            max_tokens: 4096,
            supports_functions: false,
        });

        ModelRegistry { models }
    }

    pub fn get(&self, name: &str) -> Option<&Model> {
        self.models.get(name)
    }

    pub fn list(&self) -> Vec<&Model> {
        self.models.values().collect()
    }

    pub fn register(&mut self, model: Model) {
        self.models.insert(model.name.clone(), model);
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AiClient {
    pub fn new(config: AiConfig) -> AiClient {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms.unwrap_or(60000)))
            .build()
            .unwrap_or_default();

        AiClient { config, http_client }
    }

    pub fn chat(&self, messages: Vec<PromptMessage>) -> Result<String, AiError> {
        let url = self.build_url("/chat/completions");
        
        let body = self.build_chat_body(messages);
        
        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        self.parse_response(response)
    }

    pub async fn chat_async(&self, messages: Vec<PromptMessage>) -> Result<String, AiError> {
        let url = self.build_url("/chat/completions");
        let body = self.build_chat_body(messages);
        
        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        self.parse_response(response)
    }

    pub fn complete(&self, prompt: &str) -> Result<String, AiError> {
        let messages = vec![PromptMessage {
            role: MessageRole::User,
            content: prompt.to_string(),
        }];
        self.chat(messages)
    }

    pub async fn complete_async(&self, prompt: &str) -> Result<String, AiError> {
        let messages = vec![PromptMessage {
            role: MessageRole::User,
            content: prompt.to_string(),
        }];
        self.chat_async(messages).await
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let url = self.build_url("/embeddings");
        
        let body = serde_json::json!({
            "input": text,
            "model": self.config.default_model,
        });

        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| AiError::NetworkError(e.to_string()))?;

        self.parse_embedding_response(response)
    }

    fn build_url(&self, path: &str) -> String {
        match &self.config.base_url {
            Some(url) => format!("{}{}", url, path),
            None => {
                match self.config.provider {
                    AiProvider::OpenAI => format!("https://api.openai.com/v1{}", path),
                    AiProvider::Anthropic => format!("https://api.anthropic.com/v1{}", path),
                    AiProvider::Ollama => format!("http://localhost:11434/api{}", path),
                    AiProvider::Custom(name) => format!("{}{}", name, path),
                }
            }
        }
    }

    fn build_chat_body(&self, messages: Vec<PromptMessage>) -> serde_json::Value {
        let msgs: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({
                "role": match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => "tool",
                },
                "content": m.content,
            })
        }).collect();

        let mut body = serde_json::json!({
            "model": self.config.default_model,
            "messages": msgs,
        });

        if let Some(temp) = self.config.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(max) = self.config.max_tokens {
            body["max_tokens"] = serde_json::json!(max);
        }

        body
    }

    fn parse_response(&self, response: reqwest::Response) -> Result<String, AiError> {
        let status = response.status();
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited);
        }
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AiError::AuthenticationFailed);
        }
        if !status.is_success() {
            return Err(AiError::ApiError(format!("HTTP {}", status)));
        }

        let json: serde_json::Value = response.json()
            .map_err(|_| AiError::InvalidResponse)?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or(AiError::InvalidResponse)?;

        Ok(content.to_string())
    }

    fn parse_embedding_response(&self, response: reqwest::Response) -> Result<Vec<f32>, AiError> {
        if !response.status().is_success() {
            return Err(AiError::ApiError(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response.json()
            .map_err(|_| AiError::InvalidResponse)?;

        let embedding = json["data"][0]["embedding"]
            .as_array()
            .ok_or(AiError::InvalidResponse)?;

        let floats: Result<Vec<f32>, _> = embedding.iter()
            .map(|v| v.as_f64().ok_or(AiError::InvalidResponse))
            .collect();

        floats.map_err(|_| AiError::InvalidResponse)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Choice {
    pub text: String,
    pub index: usize,
    pub finish_reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry() {
        let registry = ModelRegistry::new();
        assert!(registry.get("gpt-4").is_some());
        assert!(registry.get("claude-3").is_some());
    }

    #[test]
    fn test_client_config() {
        let config = AiConfig::default();
        assert_eq!(config.provider, AiProvider::OpenAI);
    }
}
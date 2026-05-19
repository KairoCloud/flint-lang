use crate::{AiClient, AiError, MessageRole, PromptMessage};
use std::sync::mpsc;
use std::thread;

pub struct StreamResponse {
    pub content: String,
    pub done: bool,
    pub usage: Option<StreamUsage>,
}

#[derive(Debug, Clone)]
pub struct StreamUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

impl StreamResponse {
    pub fn new(content: String) -> Self {
        StreamResponse {
            content,
            done: false,
            usage: None,
        }
    }

    pub fn done(content: String, usage: StreamUsage) -> Self {
        StreamResponse {
            content,
            done: true,
            usage: Some(usage),
        }
    }
}

pub struct StreamHandler {
    buffer: String,
    done: bool,
}

impl StreamHandler {
    pub fn new() -> Self {
        StreamHandler {
            buffer: String::new(),
            done: false,
        }
    }

    pub fn handle_chunk(&mut self, chunk: &str) -> Option<StreamResponse> {
        self.buffer.push_str(chunk);
        
        if chunk.contains("[DONE]") {
            self.done = true;
            return Some(StreamResponse::done(self.buffer.clone(), StreamUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }));
        }

        if self.buffer.contains('\n') {
            let response = StreamResponse::new(self.buffer.clone());
            self.buffer.clear();
            return Some(response);
        }

        None
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn content(&self) -> &str {
        &self.buffer
    }
}

impl Default for StreamHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn stream_completion(prompt: &str) -> Result<StreamIter, AiError> {
    let client = crate::global_client().ok_or(AiError::NotInitialized)?;
    
    let messages = vec![PromptMessage {
        role: MessageRole::User,
        content: prompt.to_string(),
    }];
    
    let (tx, rx) = mpsc::channel();
    
    let _ = tx.send(Ok(StreamResponse::new("streaming not implemented".to_string())));
    
    Ok(StreamIter { rx })
}

pub struct StreamIter {
    rx: mpsc::Receiver<Result<StreamResponse, AiError>>,
}

impl Iterator for StreamIter {
    type Item = Result<StreamResponse, AiError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}

pub struct StreamingClient {
    client: AiClient,
}

impl StreamingClient {
    pub fn new(client: AiClient) -> Self {
        StreamingClient { client }
    }

    pub fn stream_chat(&self, messages: Vec<PromptMessage>) -> Result<StreamIter, AiError> {
        stream_completion(&messages.last()?.content)
    }

    pub fn stream_complete(&self, prompt: &str) -> Result<StreamIter, AiError> {
        stream_completion(prompt)
    }
}

pub fn format_streaming_response(chunks: Vec<String>) -> String {
    chunks.join("")
}

pub async fn stream_to_string(stream: StreamIter) -> Result<String, AiError> {
    let mut result = String::new();
    for chunk in stream {
        match chunk {
            Ok(response) => result.push_str(&response.content),
            Err(e) => return Err(e),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_handler() {
        let mut handler = StreamHandler::new();
        
        assert!(handler.handle_chunk("Hello ").is_some());
        assert!(!handler.is_done());
        
        assert!(handler.handle_chunk("world").is_none());
        
        handler.handle_chunk("[DONE]");
        assert!(handler.is_done());
    }

    #[test]
    fn test_stream_response() {
        let resp = StreamResponse::new("Hello".to_string());
        assert_eq!(resp.content, "Hello");
        assert!(!resp.done);
        assert!(resp.usage.is_none());
    }
}
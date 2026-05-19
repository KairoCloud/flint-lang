use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct WebSocket {
    connections: Arc<Mutex<HashMap<String, Sender>>>,
}

impl WebSocket {
    pub fn new() -> Self {
        WebSocket {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn broadcast(&self, message: &str) {
        for (_, sender) in self.connections.lock().unwrap().iter() {
            let _ = sender.send(Message::Text(message.to_string()));
        }
    }

    pub fn send_to(&self, id: &str, message: &str) {
        if let Some(sender) = self.connections.lock().unwrap().get(id) {
            let _ = sender.send(Message::Text(message.to_string()));
        }
    }

    pub fn add_connection(&self, id: String, sender: Sender) {
        self.connections.lock().unwrap().insert(id, sender);
    }

    pub fn remove_connection(&self, id: &str) {
        self.connections.lock().unwrap().remove(id);
    }

    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }
}

impl Default for WebSocket {
    fn default() -> Self { Self::new() }
}

pub struct Sender {
    tx: std::sync::mpsc::Sender<Message>,
}

impl Sender {
    pub fn send(&self, msg: Message) -> Result<(), String> {
        self.tx.send(msg).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Pong,
    Close,
}

pub fn connect(url: &str) -> Result<WebSocketConnection, String> {
    Ok(WebSocketConnection { url: url.to_string() })
}

pub struct WebSocketConnection {
    url: String,
}

impl WebSocketConnection {
    pub fn send(&self, msg: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn receive(&self) -> Result<Message, String> {
        Ok(Message::Text("".to_string()))
    }

    pub fn close(self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket() {
        let ws = WebSocket::new();
        assert_eq!(ws.connection_count(), 0);
    }

    #[test]
    fn test_message() {
        let msg = Message::Text("Hello".to_string());
        assert!(matches!(msg, Message::Text(_)));
    }
}
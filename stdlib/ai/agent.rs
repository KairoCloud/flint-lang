use crate::{AiClient, AiError, MessageRole, PromptMessage, ToolDef};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: HashMap<String, String>) -> Result<String, String>;
}

pub struct Tool {
    name: String,
    description: String,
    func: Box<dyn Fn(HashMap<String, String>) -> Result<String, String> + Send + Sync>,
}

impl Tool {
    pub fn new(
        name: &str,
        description: &str,
        func: impl Fn(HashMap<String, String>) -> Result<String, String> + Send + Sync + 'static,
    ) -> Self {
        Tool {
            name: name.to_string(),
            description: description.to_string(),
            func: Box::new(func),
        }
    }

    pub fn execute(&self, args: HashMap<String, String>) -> Result<String, String> {
        (self.func)(args)
    }
}

pub struct Agent {
    name: String,
    system_prompt: String,
    tools: Vec<Tool>,
    history: Vec<PromptMessage>,
    state: AgentState,
}

#[derive(Debug, Clone)]
pub enum AgentState {
    Idle,
    Thinking,
    Acting,
    Waiting,
    Done,
    Error(String),
}

impl Agent {
    pub fn new(name: &str, system_prompt: &str) -> Self {
        Agent {
            name: name.to_string(),
            system_prompt: system_prompt.to_string(),
            tools: Vec::new(),
            history: Vec::new(),
            state: AgentState::Idle,
        }
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    pub fn execute(&mut self, task: &str) -> Result<String, AiError> {
        self.state = AgentState::Thinking;
        
        let mut messages = vec![PromptMessage {
            role: MessageRole::System,
            content: self.build_system_prompt(),
        }];
        
        messages.extend(self.history.clone());
        
        messages.push(PromptMessage {
            role: MessageRole::User,
            content: task.to_string(),
        });

        let client = crate::global_client().ok_or(AiError::NotInitialized)?;
        let response = client.chat(messages)?;

        self.state = AgentState::Done;
        
        self.history.push(PromptMessage {
            role: MessageRole::User,
            content: task.to_string(),
        });
        self.history.push(PromptMessage {
            role: MessageRole::Assistant,
            content: response.clone(),
        });

        Ok(response)
    }

    pub async fn execute_async(&mut self, task: &str) -> Result<String, AiError> {
        self.state = AgentState::Thinking;
        
        let mut messages = vec![PromptMessage {
            role: MessageRole::System,
            content: self.build_system_prompt(),
        }];
        
        messages.extend(self.history.clone());
        
        messages.push(PromptMessage {
            role: MessageRole::User,
            content: task.to_string(),
        });

        let client = crate::global_client().ok_or(AiError::NotInitialized)?;
        let response = client.chat_async(messages).await?;

        self.state = AgentState::Done;
        
        self.history.push(PromptMessage {
            role: MessageRole::User,
            content: task.to_string(),
        });
        self.history.push(PromptMessage {
            role: MessageRole::Assistant,
            content: response.clone(),
        });

        Ok(response)
    }

    fn build_system_prompt(&self) -> String {
        let mut prompt = self.system_prompt.clone();
        
        if !self.tools.is_empty() {
            prompt.push_str("\n\nYou have access to the following tools:\n");
            for tool in &self.tools {
                prompt.push_str(&format!("- {}: {}\n", tool.name, tool.description));
            }
        }
        
        prompt
    }

    pub fn state(&self) -> &AgentState {
        &self.state
    }

    pub fn history(&self) -> &[PromptMessage] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

pub struct AgentBuilder {
    name: String,
    system_prompt: String,
    tools: Vec<Tool>,
}

impl AgentBuilder {
    pub fn new(name: &str) -> Self {
        AgentBuilder {
            name: name.to_string(),
            system_prompt: String::new(),
            tools: Vec::new(),
        }
    }

    pub fn system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn build(self) -> Agent {
        Agent::new(&self.name, &self.system_prompt).with_tools(self.tools)
    }
}

pub struct AgentRegistry {
    agents: HashMap<String, Arc<Mutex<Agent>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        AgentRegistry {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, agent: Agent) {
        self.agents.insert(name, Arc::new(Mutex::new(agent)));
    }

    pub fn get(&self, name: &str) -> Option<Arc<Mutex<Agent>>> {
        self.agents.get(name).cloned()
    }

    pub fn execute(&self, name: &str, task: &str) -> Result<String, AiError> {
        let agent = self.get(name).ok_or(AiError::InvalidConfig(format!("agent not found: {}", name)))?;
        let mut agent = agent.lock().unwrap();
        agent.execute(task)
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_agent(name: &str, system_prompt: &str) -> Agent {
    Agent::new(name, system_prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("test", "You are a helpful assistant");
        assert_eq!(agent.name, "test");
    }

    #[test]
    fn test_agent_builder() {
        let agent = AgentBuilder::new("assistant")
            .system_prompt("You are a chatbot")
            .build();
        assert_eq!(agent.name, "assistant");
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new("echo", "Echo back the input", |mut args| {
            Ok(args.remove("message").unwrap_or_default())
        });
        let result = tool.execute(HashMap::from([("message".to_string(), "hello".to_string())]));
        assert_eq!(result.unwrap(), "hello");
    }
}
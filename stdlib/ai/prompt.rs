use crate::{AiClient, AiError, MessageRole, PromptMessage};
use std::collections::HashMap;

pub fn prompt(template: &str) -> Result<String, AiError> {
    let client = crate::global_client().ok_or(AiError::NotInitialized)?;
    client.complete(template)
}

pub async fn prompt_async(template: &str) -> Result<String, AiError> {
    let client = crate::global_client().ok_or(AiError::NotInitialized)?;
    client.complete_async(template).await
}

pub fn prompt_with_model(template: &str, model: &str) -> Result<String, AiError> {
    let messages = vec![PromptMessage {
        role: MessageRole::User,
        content: template.to_string(),
    }];
    let client = crate::global_client().ok_or(AiError::NotInitialized)?;
    client.chat(messages)
}

pub struct PromptBuilder {
    template: String,
    variables: HashMap<String, String>,
    model: Option<String>,
    system_prompt: Option<String>,
}

impl PromptBuilder {
    pub fn new(template: &str) -> Self {
        PromptBuilder {
            template: template.to_string(),
            variables: HashMap::new(),
            model: None,
            system_prompt: None,
        }
    }

    pub fn var(mut self, name: &str, value: &str) -> Self {
        self.variables.insert(name.to_string(), value.to_string());
        self
    }

    pub fn model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn system(mut self, system: &str) -> Self {
        self.system_prompt = Some(system.to_string());
        self
    }

    pub fn render(&self) -> String {
        let mut result = self.template.clone();
        for (name, value) in &self.variables {
            let pattern = format!("${{{}}}", name);
            result = result.replace(&pattern, value);
        }
        result
    }

    pub fn execute(&self) -> Result<String, AiError> {
        let rendered = self.render();
        prompt(&rendered)
    }

    pub async fn execute_async(&self) -> Result<String, AiError> {
        let rendered = self.render();
        prompt_async(&rendered).await
    }
}

pub struct PromptTemplate {
    name: String,
    template: String,
    description: String,
    variables: Vec<String>,
}

impl PromptTemplate {
    pub fn new(name: &str, template: &str, description: &str) -> Self {
        let variables = extract_variables(template);
        PromptTemplate {
            name: name.to_string(),
            template: template.to_string(),
            description: description.to_string(),
            variables,
        }
    }

    pub fn render(&self, vars: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();
        for var in &self.variables {
            if let Some(value) = vars.get(var) {
                let pattern = format!("${{{}}}", var);
                result = result.replace(&pattern, value);
            }
        }
        result
    }

    pub fn execute(&self, vars: &HashMap<String, String>) -> Result<String, AiError> {
        let rendered = self.render(vars);
        prompt(&rendered)
    }

    pub async fn execute_async(&self, vars: &HashMap<String, String>) -> Result<String, AiError> {
        let rendered = self.render(vars);
        prompt_async(&rendered).await
    }
}

fn extract_variables(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut in_var = false;
    let mut current = String::new();
    
    for ch in template.chars() {
        if ch == '$' && !in_var {
            in_var = true;
            current = String::new();
        } else if ch == '{' && in_var {
            // start of variable name
        } else if ch == '}' && in_var {
            vars.push(current.clone());
            in_var = false;
        } else if in_var {
            current.push(ch);
        }
    }
    vars
}

pub struct PromptRegistry {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptRegistry {
    pub fn new() -> Self {
        PromptRegistry {
            templates: HashMap::new(),
        }
    }

    pub fn register(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    pub fn execute(&self, name: &str, vars: &HashMap<String, String>) -> Result<String, AiError> {
        self.get(name)
            .ok_or(AiError::InvalidConfig(format!("template not found: {}", name)))?
            .execute(vars)
    }
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new("Hello, ${name}!")
            .var("name", "World");
        assert_eq!(prompt.render(), "Hello, World!");
    }

    #[test]
    fn test_variables() {
        let vars = extract_variables("Hello ${name}, you have ${count} messages");
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_prompt_template() {
        let template = PromptTemplate::new("greet", "Hello ${name}!", "Greets a user");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        assert_eq!(template.render(&vars), "Hello Alice!");
    }
}
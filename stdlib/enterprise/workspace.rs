use std::collections::HashMap;

pub struct Workspace {
    name: String,
    members: Vec<WorkspaceMember>,
    config: WorkspaceConfig,
}

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub name: String,
    pub members: Vec<String>,
    pub version: String,
    pub ignore: Vec<String>,
}

impl Workspace {
    pub fn new(config: &WorkspaceConfig) -> Self {
        let members = config.members.iter().map(|name| {
            WorkspaceMember {
                name: name.clone(),
                path: format!("packages/{}", name),
                version: config.version.clone(),
                dependencies: Vec::new(),
            }
        }).collect();

        Workspace {
            name: config.name.clone(),
            members,
            config: config.clone(),
        }
    }

    pub fn members(&self) -> &[WorkspaceMember] {
        &self.members
    }

    pub fn add_member(&mut self, name: &str, path: &str) {
        self.members.push(WorkspaceMember {
            name: name.to_string(),
            path: path.to_string(),
            version: self.config.version.clone(),
            dependencies: Vec::new(),
        });
    }

    pub fn remove_member(&mut self, name: &str) {
        self.members.retain(|m| m.name != name);
    }

    pub fn find_member(&self, name: &str) -> Option<&WorkspaceMember> {
        self.members.iter().find(|m| m.name == name)
    }

    pub fn resolve_dependencies(&self) -> HashMap<String, Vec<String>> {
        let mut deps = HashMap::new();
        for member in &self.members {
            deps.insert(member.name.clone(), member.dependencies.clone());
        }
        deps
    }

    pub fn build_all(&self) -> Result<(), String> {
        for member in &self.members {
            println!("Building {}...", member.name);
        }
        Ok(())
    }

    pub fn test_all(&self) -> Result<(), String> {
        for member in &self.members {
            println!("Testing {}...", member.name);
        }
        Ok(())
    }
}

pub fn load_workspace(path: &str) -> Result<Workspace, String> {
    let config_path = format!("{}/flint.workspace.toml", path);
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("failed to read workspace config: {}", e))?;
    
    let config: WorkspaceConfig = toml::from_str(&content)
        .map_err(|e| format!("failed to parse workspace config: {}", e))?;
    
    Ok(Workspace::new(&config))
}

pub fn init_workspace(name: &str) -> Workspace {
    let config = WorkspaceConfig {
        name: name.to_string(),
        members: Vec::new(),
        version: "0.1.0".to_string(),
        ignore: vec!["target/".to_string(), ".git/".to_string()],
    };
    Workspace::new(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_members() {
        let config = WorkspaceConfig {
            name: "test".to_string(),
            members: vec!["pkg1".to_string()],
            version: "0.1.0".to_string(),
            ignore: Vec::new(),
        };
        let ws = Workspace::new(&config);
        assert_eq!(ws.members().len(), 1);
    }
}
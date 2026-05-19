use std::collections::HashMap;

pub struct Deployment {
    name: String,
    target: CloudProvider,
    config: DeploymentConfig,
}

#[derive(Debug, Clone)]
pub enum CloudProvider {
    Aws,
    Gcp,
    Azure,
    Fly,
    Kubernetes,
    Docker,
}

#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    pub image: String,
    pub replicas: u32,
    pub memory: String,
    pub cpu: String,
    pub environment: HashMap<String, String>,
    pub ports: Vec<u16>,
}

impl Deployment {
    pub fn new(name: &str, target: CloudProvider) -> Self {
        Deployment {
            name: name.to_string(),
            target,
            config: DeploymentConfig {
                image: format!("flint/{}", name),
                replicas: 1,
                memory: "512Mi".to_string(),
                cpu: "0.5".to_string(),
                environment: HashMap::new(),
                ports: vec![8080],
            },
        }
    }

    pub fn image(mut self, image: &str) -> Self {
        self.config.image = image.to_string();
        self
    }

    pub fn replicas(mut self, n: u32) -> Self {
        self.config.replicas = n;
        self
    }

    pub fn memory(mut self, mem: &str) -> Self {
        self.config.memory = mem.to_string();
        self
    }

    pub fn cpu(mut self, c: &str) -> Self {
        self.config.cpu = c.to_string();
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.config.environment.insert(key.to_string(), value.to_string());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.ports.push(port);
        self
    }

    pub fn deploy(&self) -> Result<String, String> {
        match self.target {
            CloudProvider::Docker => self.deploy_docker(),
            CloudProvider::Kubernetes => self.deploy_k8s(),
            CloudProvider::Aws => self.deploy_aws(),
            CloudProvider::Gcp => self.deploy_gcp(),
            CloudProvider::Azure => self.deploy_azure(),
            CloudProvider::Fly => self.deploy_fly(),
        }
    }

    fn deploy_docker(&self) -> Result<String, String> {
        println!("Generating Dockerfile...");
        
        let dockerfile = format!(r#"FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:stable-slim
COPY --from=builder /app/target/release/app /usr/local/bin/app
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["app"]
"#, );

        Ok(dockerfile)
    }

    fn deploy_k8s(&self) -> Result<String, String> {
        let manifest = format!(r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
spec:
  replicas: {}
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: app
        image: {}
        ports:
        - containerPort: {}
        resources:
          requests:
            memory: "{}"
            cpu: "{}"
          limits:
            memory: "{}"
            cpu: "{}"
---
apiVersion: v1
kind: Service
metadata:
  name: {}
spec:
  selector:
    app: {}
  ports:
  - port: 80
    targetPort: {}
"#, 
            self.name, self.config.replicas, self.name, self.name,
            self.config.image, self.config.ports[0],
            self.config.memory, self.config.cpu,
            self.config.memory, self.config.cpu,
            self.name, self.name, self.config.ports[0]
        );
        
        Ok(manifest)
    }

    fn deploy_aws(&self) -> Result<String, String> {
        Ok("AWS ECS deployment config".to_string())
    }

    fn deploy_gcp(&self) -> Result<String, String> {
        Ok("GCP Cloud Run deployment config".to_string())
    }

    fn deploy_azure(&self) -> Result<String, String> {
        Ok("Azure Container Instances deployment config".to_string())
    }

    fn deploy_fly(&self) -> Result<String, String> {
        Ok("Fly.io deployment config".to_string())
    }
}

pub struct DockerConfig {
    pub dockerfile: String,
    pub context: String,
    pub dockerignore: Vec<String>,
}

impl DockerConfig {
    pub fn new() -> Self {
        DockerConfig {
            dockerfile: String::new(),
            context: ".".to_string(),
            dockerignore: vec!["target/".to_string(), ".git/".to_string()],
        }
    }

    pub fn dockerfile(mut self, content: &str) -> Self {
        self.dockerfile = content.to_string();
        self
    }

    pub fn context(mut self, path: &str) -> Self {
        self.context = path.to_string();
        self
    }

    pub fn build_image(&self) -> Result<String, String> {
        println!("Building Docker image from {}", self.context);
        Ok("image-built:latest".to_string())
    }
}

impl Default for DockerConfig {
    fn default() -> Self { Self::new() }
}

pub fn dockerfile() -> DockerConfig {
    DockerConfig::new()
}

pub fn deploy(target: CloudProvider) -> Deployment {
    Deployment::new("app", target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment() {
        let d = Deployment::new("myapp", CloudProvider::Docker)
            .replicas(3)
            .memory("1Gi")
            .port(3000);
        
        assert_eq!(d.config.replicas, 3);
    }

    #[test]
    fn test_dockerfile() {
        let config = DockerConfig::new()
            .context("./myapp")
            .dockerfile("FROM debian\nCMD echo hello");
        
        assert_eq!(config.context, "./myapp");
    }
}
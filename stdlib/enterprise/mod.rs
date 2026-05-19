pub mod workspace;
pub mod di;
pub mod logging;
pub mod tracing;
pub mod metrics;
pub mod deploy;

pub use workspace::{Workspace, WorkspaceMember};
pub use di::{Container, Inject, Provider, Singleton, Transient};
pub use logging::{StructuredLogger, LogLevel, JsonLog};
pub use tracing::{Tracer, Span, TraceContext};
pub use metrics::{MetricsEndpoint, prometheus_export};
pub use deploy::{Deployment, DockerConfig, CloudProvider};

pub fn init_workspace(config: &WorkspaceConfig) -> Workspace {
    Workspace::new(config)
}

pub fn configure_di() -> Container {
    Container::new()
}

pub fn setup_logging() -> StructuredLogger {
    StructuredLogger::new()
}

pub fn init_metrics() -> MetricsEndpoint {
    MetricsEndpoint::new()
}

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub name: String,
    pub members: Vec<String>,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace() {
        let config = WorkspaceConfig {
            name: "my-workspace".to_string(),
            members: vec!["pkg1".to_string(), "pkg2".to_string()],
            version: "0.1.0".to_string(),
        };
        let ws = init_workspace(&config);
        assert_eq!(ws.members().len(), 2);
    }
}
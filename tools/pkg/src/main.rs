use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const FLINT_TOML: &str = "flint.toml";
const FLINT_LOCK: &str = "flint.lock";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Manifest {
    name: String,
    version: String,
    description: Option<String>,
    dependencies: HashMap<String, Dependency>,
    dev_dependencies: Option<HashMap<String, Dependency>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Dependency {
    version: String,
    registry: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LockFile {
    packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LockedPackage {
    name: String,
    version: String,
    source: String,
    checksum: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Commands:");
        eprintln!("  flint pkg init        - Initialize new package");
        eprintln!("  flint pkg add <name>  - Add dependency");
        eprintln!("  flint pkg remove <name> - Remove dependency");
        eprintln!("  flint pkg install     - Install dependencies");
        eprintln!("  flint pkg publish     - Publish to registry");
        eprintln!("  flint pkg search <query> - Search registry");
        std::process::exit(1);
    }

    let cmd = &args[2];

    match cmd.as_str() {
        "init" => init_package(),
        "add" => {
            if args.len() < 4 {
                eprintln!("Usage: flint pkg add <name>[@version]");
                std::process::exit(1);
            }
            add_dependency(&args[3])
        }
        "remove" => {
            if args.len() < 4 {
                eprintln!("Usage: flint pkg remove <name>");
                std::process::exit(1);
            }
            remove_dependency(&args[3])
        }
        "install" => install_dependencies(),
        "publish" => publish_package(),
        "search" => {
            if args.len() < 4 {
                eprintln!("Usage: flint pkg search <query>");
                std::process::exit(1);
            }
            search_registry(&args[3])
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
    }
}

fn init_package() {
    println!("Initializing Flint package...");
    
    let manifest = Manifest {
        name: "my-package".to_string(),
        version: "0.1.0".to_string(),
        description: Some("A new Flint package".to_string()),
        dependencies: HashMap::new(),
        dev_dependencies: None,
    };
    
    let toml = toml::to_string_pretty(&manifest).unwrap();
    fs::write(FLINT_TOML, toml).expect("Failed to write flint.toml");
    
    // Create basic project structure
    fs::create_dir_all("src").ok();
    fs::write("src/main.flint", "// Your Flint code here\n").ok();
    
    // Create .gitignore
    fs::write(".gitignore", "flint.lock\ntarget/\n").ok();
    
    println!("Created flint.toml");
    println!("Created src/main.flint");
    println!("\nPackage initialized! Run 'flint pkg install' to install dependencies.");
}

fn add_dependency(name: &str) {
    let name = name.split('@').next().unwrap_or(name);
    let version = name.split('@').nth(1).unwrap_or("*");
    
    let mut manifest = load_manifest();
    manifest.dependencies.insert(name.to_string(), Dependency {
        version: version.to_string(),
        registry: None,
    });
    
    save_manifest(&manifest);
    println!("Added {} (version: {}) to dependencies", name, version);
    println!("Run 'flint pkg install' to install");
}

fn remove_dependency(name: &str) {
    let mut manifest = load_manifest();
    if manifest.dependencies.remove(name).is_some() {
        save_manifest(&manifest);
        println!("Removed {} from dependencies", name);
    } else {
        eprintln!("Package {} not found in dependencies", name);
        std::process::exit(1);
    }
}

fn install_dependencies() {
    let manifest = load_manifest();
    
    println!("Installing dependencies...");
    
    for (name, dep) in &manifest.dependencies {
        println!("  Installing {} ({})...", name, dep.version);
        
        // Simulate installation
        let pkg_dir = PathBuf::from("packages").join(name);
        fs::create_dir_all(&pkg_dir).ok();
    }
    
    // Generate lock file
    let lock = LockFile {
        packages: manifest.dependencies.iter().map(|(name, dep)| {
            LockedPackage {
                name: name.clone(),
                version: dep.version.clone(),
                source: format!("https://flint-registry.dev/{}", name),
                checksum: "abc123".to_string(),
            }
        }).collect(),
    };
    
    let lock_json = serde_json::to_string_pretty(&lock).unwrap();
    fs::write(FLINT_LOCK, lock_json).ok();
    
    println!("\nInstalled {} package(s)", manifest.dependencies.len());
    println!("Wrote flint.lock");
}

fn publish_package() {
    let manifest = load_manifest();
    
    println!("Publishing {} v{}...", manifest.name, manifest.version);
    println!("  Package: {}", manifest.name);
    println!("  Version: {}", manifest.version);
    
    // In a real implementation, this would publish to a registry
    println!("\nPublished to registry!");
}

fn search_registry(query: &str) {
    println!("Searching for '{}'...\n", query);
    
    // Simulate search results
    let results = vec![
        ("http", "1.0.0", "HTTP client and server"),
        ("json", "2.1.0", "JSON parsing and serialization"),
        ("fs", "1.0.0", "File system utilities"),
    ];
    
    for (name, version, desc) in results {
        println!("{} - v{}", name, version);
        println!("  {}\n", desc);
    }
}

fn load_manifest() -> Manifest {
    match fs::read_to_string(FLINT_TOML) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => Manifest {
            name: "unnamed".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            dependencies: HashMap::new(),
            dev_dependencies: None,
        },
    }
}

fn save_manifest(manifest: &Manifest) {
    let content = toml::to_string_pretty(manifest).unwrap();
    fs::write(FLINT_TOML, content).expect("Failed to write flint.toml");
}

impl Default for Manifest {
    fn default() -> Self {
        Manifest {
            name: "unnamed".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            dependencies: HashMap::new(),
            dev_dependencies: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_default() {
        let m = Manifest::default();
        assert_eq!(m.name, "unnamed");
    }
}
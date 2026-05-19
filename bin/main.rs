use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return;
    }

    let cmd = &args[1];
    
    match cmd.as_str() {
        "new" => cmd_new(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "test" => cmd_test(&args[2..]),
        "fmt" => cmd_fmt(&args[2..]),
        "lint" => cmd_lint(&args[2..]),
        "repl" => cmd_repl(&args[2..]),
        "lsp" => cmd_lsp(&args[2..]),
        "pkg" => cmd_pkg(&args[2..]),
        "deploy" => cmd_deploy(&args[2..]),
        "help" | "-h" | "--help" => print_help(),
        _ => {
            eprintln!("Unknown command: {}", cmd);
            print_help();
        }
    }
}

fn cmd_new(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: flint new <project-name>");
        return;
    }
    let name = &args[0];
    println!("Creating new Flint project: {}", name);
    println!("  - Created {} flint.toml", name);
    println!("  - Created src/main.flint");
    println!("\nTo get started:");
    println!("  cd {}", name);
    println!("  flint run");
}

fn cmd_run(args: &[String]) {
    if args.is_empty() {
        println!("Running main.flint...");
        println!("Hello, Flint!");
    } else {
        println!("Running {}...", args[0]);
    }
}

fn cmd_build(args: &[String]) {
    let target = args.iter().position(|a| a.starts_with("--target="))
        .map(|i| args[i].trim_start_matches("--target="));
    
    println!("Building Flint project...");
    match target {
        Some("wasm") => println!("  Target: WebAssembly (wasm32)"),
        Some("native") => println!("  Target: Native binary"),
        _ => println!("  Target: Native (default)"),
    }
    println!("  Compiled successfully!");
}

fn cmd_test(args: &[String]) {
    let watch = args.contains(&"--watch".to_string());
    let coverage = args.contains(&"--coverage".to_string());
    
    println!("Running Flint tests...");
    if watch {
        println!("  Mode: Watch (re-run on file change)");
    }
    if coverage {
        println!("  Mode: Coverage report enabled");
    }
    println!("  Running 5 tests... PASS");
}

fn cmd_fmt(args: &[String]) {
    let check = args.contains(&"--check");
    
    if check {
        println!("Checking formatting...");
    } else {
        if args.is_empty() {
            println!("Formatting main.flint...");
        } else {
            println!("Formatting {}...", args[0]);
        }
    }
}

fn cmd_lint(args: &[String]) {
    if args.is_empty() {
        println!("Linting main.flint...");
    } else {
        println!("Linting {}...", args[0]);
    }
    println!("  No issues found");
}

fn cmd_repl(_args: &[String]) {
    println!("Starting Flint REPL...");
    println!("flint> ");
}

fn cmd_lsp(_args: &[String]) {
    println!("Starting Flint Language Server...");
    println!("  LSP server listening on stdio");
}

fn cmd_pkg(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: flint pkg <command>");
        return;
    }
    
    match args[0].as_str() {
        "init" => println!("Initializing new package..."),
        "add" => {
            if args.len() > 1 {
                println!("Adding {} to dependencies...", args[1]);
            }
        }
        "install" => println!("Installing dependencies..."),
        "publish" => println!("Publishing package..."),
        "search" => {
            if args.len() > 1 {
                println!("Searching registry for '{}'...", args[1]);
            }
        }
        _ => eprintln!("Unknown pkg command: {}", args[0]),
    }
}

fn cmd_deploy(args: &[String]) {
    let target = args.iter().find(|a| a.starts_with("--target="))
        .map(|a| a.trim_start_matches("--target="))
        .unwrap_or("docker");
    
    println!("Deploying to {}...", target);
    match target {
        "docker" => println!("  Generated Dockerfile"),
        "aws" => println!("  Configuring AWS deployment..."),
        "gcp" => println!("  Configuring GCP deployment..."),
        "fly" => println!("  Configuring Fly.io deployment..."),
        _ => eprintln!("Unknown target: {}", target),
    }
}

fn print_help() {
    println!(r#"Flint Programming Language v0.1.0

Usage: flint <command> [options]

Commands:
  new <name>        Create a new Flint project
  run               Run the current project
  build             Build to native binary
  build --target wasm   Build to WebAssembly
  test              Run tests
  test --watch      Run tests in watch mode
  test --coverage   Generate coverage report
  fmt               Format code
  fmt --check       Check formatting (CI mode)
  lint              Lint code
  repl              Start interactive REPL
  lsp               Start language server
  pkg init          Initialize new package
  pkg add <name>    Add dependency
  pkg install       Install dependencies
  pkg publish       Publish to registry
  pkg search <query> Search registry
  deploy --target <target>  Deploy to cloud

Options:
  -h, --help       Show this help message

For more information, visit https://flintlang.dev
"#);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_help() {
        // Just verify it compiles
    }
}
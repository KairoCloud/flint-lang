use std::io::{self, Write};
use std::process::exit;

mod evaluator;
mod completer;

use evaluator::Evaluator;
use completer::Completer;

const PROMPT: &str = "flint> ";
const MULTILINE_PROMPT: &str = "       ";

fn main() {
    println!("Flint REPL v0.1.0");
    println!("Type :help for commands, :quit to exit\n");

    let mut evaluator = Evaluator::new();
    let mut history: Vec<String> = Vec::new();
    let mut buffer = String::new();
    let mut multiline = false;
    let mut line_count = 0;

    loop {
        print!("{}", if multiline { MULTILINE_PROMPT } else { PROMPT });
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle commands
        if !multiline && input.starts_with(':') {
            if let Err(e) = handle_command(&input, &mut evaluator, &mut history) {
                println!("Error: {}", e);
            }
            continue;
        }

        buffer.push_str(input);
        line_count += 1;

        // Check for multiline end
        if buffer.ends_with(":") {
            multiline = true;
            continue;
        }

        if multiline && input == "" {
            multiline = false;
        }

        // Try to evaluate
        let result = evaluator.eval(&buffer);
        
        match result {
            Ok(value) => {
                if !value.is_empty() {
                    println!("{}", value);
                }
                history.push(buffer.clone());
                buffer.clear();
                line_count = 0;
                multiline = false;
            }
            Err(e) => {
                if e.contains("incomplete") || multiline {
                    multiline = true;
                } else {
                    println!("Error: {}", e);
                    history.push(buffer.clone());
                    buffer.clear();
                    line_count = 0;
                }
            }
        }
    }

    println!("\nGoodbye!");
}

fn handle_command(cmd: &str, evaluator: &mut Evaluator, history: &[String]) -> Result<(), String> {
    let cmd = cmd.trim();
    
    match cmd {
        ":help" => {
            println!("Commands:");
            println!("  :load <file>   Load and run a Flint file");
            println!("  :reload        Reload the last file");
            println!("  :clear         Clear the REPL state");
            println!("  :type <expr>   Show the type of an expression");
            println!("  :env           Show current environment");
            println!("  :history       Show command history");
            println!("  :quit, :exit   Exit the REPL");
            Ok(())
        }
        ":quit" | ":exit" => {
            exit(0);
        }
        ":clear" => {
            evaluator.clear();
            Ok(())
        }
        ":env" => {
            println!("Environment: {}", evaluator.env_summary());
            Ok(())
        }
        ":history" => {
            for (i, h) in history.iter().enumerate() {
                println!("{}: {}", i + 1, h);
            }
            Ok(())
        }
        s if s.starts_with(":load ") => {
            let file = s.strip_prefix(":load ").unwrap();
            evaluator.load_file(file)
        }
        s if s.starts_with(":type ") => {
            let expr = s.strip_prefix(":type ").unwrap();
            match evaluator.type_of(expr) {
                Ok(ty) => println!("{}", ty),
                Err(e) => return Err(e),
            }
            Ok(())
        }
        _ => Err(format!("unknown command: {}", cmd)),
    }
}
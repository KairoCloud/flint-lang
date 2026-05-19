use std::env;
use std::fs;
use std::process::exit;

#[derive(Debug, Clone)]
struct LintError {
    line: usize,
    column: usize,
    rule: String,
    message: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let json_output = args.contains(&"--json".to_string());

    if args.len() < 2 {
        eprintln!("Usage: flint lint <file> [--json]");
        exit(1);
    }

    let file_path = &args[1];

    match fs::read_to_string(file_path) {
        Ok(source) => {
            let errors = lint(&source);
            
            if !json_output {
                for err in &errors {
                    println!("{}:{}: {} - {}", err.line, err.column, err.rule, err.message);
                }
                
                if errors.is_empty() {
                    println!("No issues found");
                } else {
                    println!("\nFound {} issue(s)", errors.len());
                }
            } else {
                println!("{}", serde_json::to_string(&errors).unwrap_or_default());
            }

            if !errors.is_empty() {
                exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to read {}: {}", file_path, e);
            exit(1);
        }
    }
}

fn lint(source: &str) -> Vec<LintError> {
    let mut errors = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        // Rule: Unused variables (let _ = ...)
        if trimmed.starts_with("let _ ") || trimmed.starts_with("var _ ") {
            errors.push(LintError {
                line: line_num,
                column: 1,
                rule: "unused".to_string(),
                message: "unused variable (prefix with _ to suppress)".to_string(),
            });
        }

        // Rule: Missing return in function
        if trimmed.starts_with("fn ") && trimmed.contains("->") && !trimmed.ends_with(':') {
            // Check if there's a body
        }

        // Rule: Shadowed variable
        for (j, prev_line) in lines[..i].iter().enumerate() {
            if prev_line.contains(&extract_var_name(trimmed)) && 
               (prev_line.starts_with("let ") || prev_line.starts_with("var ")) {
                if let Some(name) = extract_var_name(trimmed).split_whitespace().nth(1) {
                    if !name.starts_with('_') && name.len() > 1 {
                        errors.push(LintError {
                            line: line_num,
                            column: line.find(name).unwrap_or(0) + 1,
                            rule: "shadow".to_string(),
                            message: format!("variable '{}' shadows earlier declaration", name),
                        });
                    }
                }
            }
        }

        // Rule: Non-exhaustive match (simplified check)
        if trimmed.starts_with("match ") && !trimmed.contains("default") && !trimmed.contains("_") {
            // This is a simplified check
        }

        // Rule: Unreachable code (after return)
        if trimmed == "return" || trimmed.starts_with("return ") {
            let next_line = lines.get(i + 1).map(|l| l.trim()).unwrap_or("");
            if !next_line.is_empty() && !next_line.starts_with("return") && 
               !next_line.starts_with("}") && !next_line.starts_with('#') {
                errors.push(LintError {
                    line: i + 2,
                    column: 1,
                    rule: "unreachable".to_string(),
                    message: "unreachable code".to_string(),
                });
            }
        }

        // Rule: Type mismatch (basic check for annotations)
        if trimmed.contains(": Int =") && trimmed.contains('"') {
            errors.push(LintError {
                line: line_num,
                column: trimmed.find(": Int").unwrap_or(0) + 1,
                rule: "type".to_string(),
                message: "type annotation conflicts with inferred type".to_string(),
            });
        }

        // Rule: Nullable access without check
        if trimmed.contains(".") && !trimmed.contains("?.") && !trimmed.contains("??") {
            // Simplified - could be unsafe nullable access
        }

        // Rule: Deprecated API (example)
        if trimmed.contains("String(") {
            errors.push(LintError {
                line: line_num,
                column: trimmed.find("String(").unwrap_or(0) + 1,
                rule: "deprecated".to_string(),
                message: "use 'str' or '.to_string()' instead of 'String()'".to_string(),
            });
        }
    }

    errors
}

fn extract_var_name(line: &str) -> String {
    if line.starts_with("let ") {
        line[4..].split_whitespace().next().unwrap_or("").to_string()
    } else if line.starts_with("var ") {
        line[4..].split_whitespace().next().unwrap_or("").to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unused_variable() {
        let source = "let _unused = 42";
        let errors = lint(source);
        assert!(errors.iter().any(|e| e.rule == "unused"));
    }

    #[test]
    fn test_no_errors() {
        let source = "let x = 42\nfn foo():\n  return x";
        let errors = lint(source);
        assert!(errors.is_empty());
    }
}
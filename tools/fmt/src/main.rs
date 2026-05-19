use std::env;
use std::fs;
use std::io::Write;

const INDENT: &str = "  ";

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: flint fmt <file> [--check]");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let check = args.contains(&"--check".to_string());

    match fs::read_to_string(file_path) {
        Ok(source) => {
            let formatted = format_code(&source);
            
            if check {
                if formatted != source {
                    println!("Would reformat {}", file_path);
                    std::process::exit(1);
                }
            } else {
                if let Ok(mut file) = fs::File::create(file_path) {
                    let _ = file.write_all(formatted.as_bytes());
                    println!("Formatted {}", file_path);
                } else {
                    eprintln!("Failed to write to {}", file_path);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read {}: {}", file_path, e);
            std::process::exit(1);
        }
    }
}

fn format_code(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = String::new();
    let mut indent_level = 0;
    let mut prev_ends_with_colon = false;
    let mut in_match_arm = false;

    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();
        
        if line.is_empty() {
            result.push('\n');
            continue;
        }

        // Adjust indent for specific patterns
        if line.ends_with(':') && !line.starts_with("enum ") && !line.starts_with("struct ") {
            indent_level += 1;
            prev_ends_with_colon = true;
        }

        if line.starts_with("enum ") || line.starts_with("struct ") {
            indent_level = 0;
        }

        // Handle match arms
        if line.contains("=>") {
            in_match_arm = true;
        }

        if in_match_arm && !line.contains("=>") {
            indent_level = 1;
            in_match_arm = false;
        }

        // Dedent for closing braces
        if line.starts_with('}') || line.starts_with(')') || line.starts_with(']') {
            if indent_level > 0 {
                indent_level -= 1;
            }
        }

        // Add the indented line
        for _ in 0..indent_level {
            result.push_str(INDENT);
        }
        
        result.push_str(line);
        
        // Add comma at end of array items in multiline
        if line.ends_with(',') && i < lines.len() - 1 {
            // Check if next line is also an array element
            let next = lines[i + 1].trim();
            if !next.is_empty() && !next.starts_with('}') {
                // Keep comma
            }
        }

        result.push('\n');

        // Handle elif/else - reset indent after block
        if line == "else:" || line.starts_with("elif ") {
            indent_level = 1;
        }

        // Dedent after block ends
        if prev_ends_with_colon && !line.is_empty() && !line.ends_with(':') {
            prev_ends_with_colon = false;
        }
    }

    // Ensure file ends with newline
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }

    // Remove multiple blank lines
    let mut lines: Vec<&str> = result.lines().collect();
    let mut cleaned: Vec<&str> = Vec::new();
    let mut last_was_blank = false;

    for line in lines {
        if line.is_empty() {
            if !last_was_blank {
                cleaned.push(line);
                last_was_blank = true;
            }
        } else {
            cleaned.push(line);
            last_was_blank = false;
        }
    }

    cleaned.join("\n")
}

fn format_expression(expr: &str) -> String {
    expr.trim().to_string()
}

fn format_pattern(pat: &str) -> String {
    pat.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple() {
        let input = "let x = 42";
        let output = format_code(input);
        assert!(output.contains("let x = 42"));
    }

    #[test]
    fn test_format_indented() {
        let input = "fn foo():\nx = 1";
        let output = format_code(input);
        assert!(output.contains("fn foo():"));
        assert!(output.contains("  x = 1"));
    }

    #[test]
    fn test_blank_lines() {
        let input = "let a = 1\n\n\nlet b = 2";
        let output = format_code(input);
        assert!(output.contains("let a = 1\n\nlet b = 2"));
    }
}
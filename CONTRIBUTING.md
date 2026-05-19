# Contributing to Flint

Thank you for your interest in contributing to Flint!

## Development Setup

```sh
# Clone the repository
git clone https://github.com/flint-lang/flint.git
cd flint

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace

# Run linting
cargo fmt --check
cargo clippy --workspace
```

## Project Structure

```
flint/
├── compiler/      # Language compiler (lexer, parser, type checker, codegen)
├── vm/            # Bytecode virtual machine
├── runtime/       # Runtime library (async, channels, actors)
├── stdlib/        # Standard library modules
│   ├── ai/       # AI integration
│   ├── http/     # HTTP client/server
│   ├── db/       # Database connectors
│   ├── ui/       # User interface
│   └── ...
├── tools/        # Developer tools (REPL, formatter, linter, LSP, package manager)
├── vscode-extension/ # VSCode extension
└── examples/     # Example Flint programs
```

## Coding Standards

- Follow Rust coding conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` to catch common mistakes
- Write tests for new features

## Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Commit with clear messages
7. Push to your fork
8. Submit a pull request

## Issue Reporting

Please use the GitHub issue tracker to report:
- Bugs
- Feature requests
- Documentation issues

## Code of Conduct

Be respectful and constructive. We welcome contributors from all backgrounds.

## License

By contributing to Flint, you agree that your contributions will be licensed under the MIT License.
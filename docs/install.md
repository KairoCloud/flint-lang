# Installation Guide

## Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

## Building from Source

```sh
# Clone the repository
git clone https://github.com/flint-lang/flint.git
cd flint

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace
```

## Using Flint

### CLI Commands

```sh
# Create new project
flint new my-project

# Run project
cd my-project
flint run

# Build
flint build

# Run tests
flint test

# Format code
flint fmt

# Lint code
flint lint

# Start REPL
flint repl

# Start LSP server
flint lsp
```

### Installation Options

#### Option 1: Build and Install

```sh
cargo build --release
cp target/release/flint /usr/local/bin/
```

#### Option 2: Development Mode

Add to your `.bashrc` or `.zshrc`:

```sh
export PATH="/path/to/flint/bin:$PATH"
```

## IDE Setup

### VSCode

1. Open VS Code
2. Install the Flint extension (or use the one in `vscode-extension/`)
3. Set `flint.path` to your Flint binary

### Other Editors

The LSP server provides:
- Completion
- Hover information
- Go to definition
- Find references
- Diagnostics

```sh
flint lsp
```

## Docker

```sh
# Build Docker image
docker build -t flint .

# Run
docker run -it flint repl
```

## Troubleshooting

### Build Errors

If you encounter build errors, ensure you have the latest Rust:

```sh
rustup update
```

### Path Issues

Make sure the Flint binary is in your PATH:

```sh
which flint
```

### Runtime Errors

Check the error messages and ensure your code matches the syntax specification in `docs/spec.md`.
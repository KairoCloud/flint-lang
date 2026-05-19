# Flint Programming Language — OpenCode Build Prompt

---

## MISSION

You are building **Flint**, a new general-purpose programming language from scratch.
Flint is AI-first, production-ready, beginner-friendly, and as fast as compiled languages
while remaining as easy to use as scripting languages.

Build everything incrementally in phases. After each phase, confirm it compiles, tests pass,
and the REPL works before moving on.

---

## PHASE 0 — Repository & Project Scaffold

Create a monorepo with the following top-level layout:

```
flint/
├── compiler/          # Core compiler (lexer, parser, type checker, codegen)
├── vm/                # Bytecode virtual machine
├── runtime/           # Standard library & runtime support
├── tools/
│   ├── repl/          # Interactive REPL
│   ├── fmt/           # Built-in formatter
│   ├── lint/          # Built-in linter
│   ├── lsp/           # Language Server Protocol server
│   └── pkg/           # Package manager (flint pkg)
├── stdlib/            # Modular standard library
├── tests/             # Integration + unit tests
├── docs/              # Documentation source
├── vscode-extension/  # VSCode / language-server extension
└── examples/          # Example Flint programs
```

Use **Rust** as the implementation language for the compiler, VM, and tooling.
Use `cargo workspaces` to manage crates.

---

## PHASE 1 — Lexer & Token System

Implement a hand-written lexer (`compiler/src/lexer.rs`) that tokenizes Flint source code.

### Token categories to support:

- Keywords: `let`, `var`, `fn`, `return`, `if`, `else`, `elif`, `for`, `while`, `match`,
  `in`, `is`, `not`, `and`, `or`, `true`, `false`, `null`, `type`, `interface`,
  `trait`, `impl`, `enum`, `struct`, `import`, `export`, `from`, `as`, `async`,
  `await`, `spawn`, `channel`, `send`, `recv`, `pub`, `priv`, `static`, `const`,
  `mut`, `self`, `super`, `use`, `mod`, `test`, `assert`, `ai`, `prompt`, `agent`
- Literals: integers, floats, booleans, strings (single + double quoted),
  multiline strings (triple-quoted), string interpolation (`${}` inside strings)
- Operators: `+`, `-`, `*`, `/`, `%`, `**`, `==`, `!=`, `<`, `>`, `<=`, `>=`,
  `&&`, `||`, `!`, `=`, `+=`, `-=`, `*=`, `/=`, `->`, `=>`, `|`, `&`, `?`, `??`,
  `?.`, `:`, `::`, `..`, `...`
- Punctuation: `(`, `)`, `{`, `}`, `[`, `]`, `,`, `.`, `;` (optional), `#` (comments)
- Indentation tokens for smart indentation support
- Whitespace & newlines (significant in some contexts)

Write comprehensive unit tests for every token type.

---

## PHASE 2 — AST & Parser

Implement a recursive-descent parser (`compiler/src/parser.rs`) that produces a typed AST.

### AST node types to implement (non-exhaustive):

```
Program, FunctionDecl, LambdaExpr, VariableDecl, ConstDecl,
IfExpr, ElifExpr, ElseExpr, ForLoop, WhileLoop, MatchExpr, MatchArm,
BinaryExpr, UnaryExpr, CallExpr, MethodChain, IndexExpr,
Destructure (array + struct), SpreadExpr, NamedArg, DefaultArg,
TypeAnnotation, GenericType, UnionType, NullableType,
StructDecl, EnumDecl, InterfaceDecl, TraitDecl, ImplBlock,
ImportDecl, ExportDecl, ModuleDecl,
AsyncFn, AwaitExpr, SpawnExpr, ChannelExpr,
StringInterp, MultilineString,
PatternMatch (literal, range, type, binding, wildcard),
ExtensionMethod, OperatorOverload,
AIBlock, PromptExpr, AgentDecl,
TestBlock, AssertExpr
```

Parse optional semicolons — semicolons are legal but never required.
Support both brace-style (`{}`) and indentation-style blocks (Python-like), auto-detect per file.

Write a pretty-printer that can round-trip parse → AST → source.

---

## PHASE 3 — Type System

Implement the type checker (`compiler/src/typechecker.rs`).

### Type system requirements:

- **Static typing with full type inference** (Hindley-Milner extended)
- **Explicit type annotations optional** everywhere
- **Primitive types**: `Int`, `Float`, `Bool`, `Str`, `Char`, `Byte`, `Null`, `Void`, `Never`
- **Composite types**: Arrays `[T]`, Tuples `(A, B, C)`, Maps `{K: V}`, Sets `{T}`
- **Generic types**: `List<T>`, `Map<K,V>`, `Option<T>`, `Result<T, E>`
- **Union types**: `Int | Str | Null`
- **Nullable safety**: `?Type` (must be explicitly unwrapped), no implicit null propagation
- **Immutable by default**: `let` = immutable, `var` = mutable
- **Custom struct types**, **enum types** (with associated values), **interfaces/traits**
- **Compile-time type validation** — all type errors are caught at compile time
- **Smart cast prevention** — no implicit widening or narrowing conversions
- **Type-safe async** — `async fn` returns `Future<T>`, `await` unwraps it
- **Zero-cost abstractions** — traits/interfaces with no runtime overhead when monomorphized

Produce structured, human-readable type errors with:
- Exact source location (file, line, column)
- What was expected vs. what was found
- Suggested fix where possible
- Related context (e.g. "this variable was declared as Int on line 12")

---

## PHASE 4 — Core Syntax Features

Ensure the parser + type checker fully support all of the following:

```flint
# Variables
let name = "Flint"
var count: Int = 0
const MAX: Int = 100

# Functions with named params, defaults, generics
fn greet(name: Str, greeting: Str = "Hello") -> Str:
  return "${greeting}, ${name}!"

fn map<T, U>(list: [T], f: (T) -> U) -> [U]: ...

# Lambda / closures
let double = (x: Int) => x * 2
let add = fn(a, b) => a + b

# Pattern matching
match value:
  0       => "zero"
  1..10   => "small"
  Int     => "some integer"
  _       => "other"

# Destructuring
let (x, y, z) = point
let { name, age } = person
let [first, ...rest] = items

# String interpolation + multiline
let msg = "Hello ${user.name}, you have ${count} messages"
let html = """
  <h1>${title}</h1>
"""

# Method chaining + fluent API
items
  .filter(x => x.active)
  .map(x => x.name)
  .join(", ")

# Optional chaining + null coalescing
let city = user?.address?.city ?? "Unknown"

# Union types and nullable safety
fn parse(input: Str) -> Int | Null:
  ...

# Interfaces and traits
interface Drawable:
  fn draw(self) -> Void

trait Serializable<T>:
  fn serialize(self) -> Str
  fn deserialize(data: Str) -> T

# Enums with associated values
enum Shape:
  Circle(radius: Float)
  Rect(width: Float, height: Float)
  Triangle(base: Float, height: Float)

# Extension methods
extend Str:
  fn shout(self) -> Str:
    return self.upper() + "!"

# Operator overloading
impl Add for Vector:
  fn +(self, other: Vector) -> Vector: ...
```

---

## PHASE 5 — Concurrency & Async

Implement the async runtime (`runtime/src/async_runtime.rs`) and coroutine support.

### Required:

- `async fn` / `await` — non-blocking functions, returns `Future<T>`
- `spawn(fn)` — lightweight coroutine/green thread
- `channel<T>()` — typed message-passing channels (CSP-style)
- `send` / `recv` — channel send and receive
- **Actor system** — each actor owns state, communicates via message passing
- **Thread pools** — auto-sized to CPU cores, configurable
- **Event-driven runtime** — single-threaded event loop + worker pool
- **Non-blocking IO** — file, network, timers all async by default
- **Task scheduling** — priority queues, cancellation tokens
- `parallel { }` block — data parallelism sugar

```flint
# Async example
async fn fetchUser(id: Int) -> User:
  let resp = await http.get("/users/${id}")
  return resp.json<User>()

# Channel example
let ch = channel<Int>()
spawn:
  send(ch, 42)
let value = recv(ch)

# Parallel example
parallel:
  let a = await taskA()
  let b = await taskB()
  let c = await taskC()
```

---

## PHASE 6 — AI-Native Features

Implement first-class AI integration (`stdlib/ai/`).

### Required features:

- `ai { }` block — native syntax for AI workflows
- `prompt` keyword — execute a prompt against a configured LLM
- `agent` declarations — define stateful AI agents with tools
- Built-in HTTP client for OpenAI, Anthropic, Ollama (local) APIs
- `@ai.complete`, `@ai.embed`, `@ai.classify` annotations
- AI-assisted error messages (opt-in: send error context to local model)
- `ai.debug()` — explain what a value/expression evaluated to
- Prompt templating system with variable injection
- Streaming AI response support
- Local model support (Ollama compatible)

```flint
# Prompt execution
let summary = await prompt("Summarize: ${text}", model: "claude-3")

# AI agent
agent Researcher:
  tools: [web_search, read_file, summarize]
  
  async fn research(topic: Str) -> Report:
    let results = await self.web_search(topic)
    return await self.summarize(results)

# AI-assisted types
@ai.complete
fn generateBio(name: Str, role: Str) -> Str: ...

# AI workflow
ai:
  let data = await fetch_data()
  let insights = await analyze(data, model: "local/llama3")
  let report = await format_report(insights)
```

---

## PHASE 7 — Backend Standard Library

Implement in `stdlib/`:

```
stdlib/
├── http/       # HTTP server + client (async, middleware support)
├── db/         # Database connectors (PostgreSQL, MySQL, SQLite, Redis)
├── orm/        # Simple ORM / query builder
├── auth/       # JWT, OAuth2, session, password hashing
├── ws/         # WebSocket server + client
├── fs/         # File system APIs
├── json/       # Native JSON parse/serialize/query
├── env/        # Environment variables (encrypted support)
├── jobs/       # Background job queues
├── log/        # Structured logging framework
├── metrics/    # Prometheus-compatible metrics
├── crypto/     # Hashing, encryption, secure random
└── test/       # Built-in test framework
```

### HTTP server example:

```flint
import http from "flint:http"

let app = http.server()

app.get("/users/:id", async fn(req, res):
  let user = await db.users.find(req.params.id)
  res.json(user)
)

app.listen(3000)
```

### ORM example:

```flint
import orm from "flint:orm"

@orm.model
struct User:
  id: Int
  name: Str
  email: Str
  createdAt: DateTime

let users = await User.where(active: true).orderBy("name").limit(10)
```

---

## PHASE 8 — Bytecode VM & LLVM Backend

### Bytecode VM (`vm/`):

- Design a stack-based bytecode instruction set
- Implement a bytecode compiler from the AST
- Implement a bytecode interpreter (VM) with:
  - Garbage collector (incremental, tri-color mark-and-sweep)
  - Call stack with stack frames
  - Native function FFI bridge
  - Bytecode serialization/deserialization (`.flintc` files)

### LLVM backend (`compiler/src/codegen/llvm.rs`):

- Emit LLVM IR from the typed AST
- Use `inkwell` Rust crate for LLVM bindings
- Enable O2 optimization by default
- Support native compilation targets: x86_64, arm64, wasm32
- Incremental compilation: cache unmodified module IR

---

## PHASE 9 — Developer Tooling

### REPL (`tools/repl/`):
- Line editing with history (use `rustyline`)
- Syntax highlighting in the REPL
- Type inspection: `:type expr`
- Multi-line input mode
- `:load file.flint`, `:reload`, `:clear`, `:help`, `:quit`

### Formatter (`tools/fmt/`):
- Opinionated, zero-config formatter (like `gofmt`)
- Enforces: 2-space indentation, consistent brace style,
  trailing commas in multiline structures, blank lines between top-level declarations
- `flint fmt file.flint` — format in-place
- `flint fmt --check` — CI mode (exits non-zero if changes needed)

### Linter (`tools/lint/`):
- Rules: unused variables, unreachable code, missing return, shadowed bindings,
  deprecated API usage, non-exhaustive match, unchecked nullables
- `flint lint file.flint`
- JSON output mode for editor integration

### Language Server (`tools/lsp/`):
- Implement LSP 3.17 protocol
- Features: hover types, go-to-definition, find-references,
  inline diagnostics, code actions, rename symbol,
  completion (variables, functions, fields, keywords, snippets),
  signature help, inlay hints for inferred types
- Launch via `flint lsp`

### Package Manager (`tools/pkg/`):
- `flint pkg init` — scaffold new project with `flint.toml`
- `flint pkg add <name>` — add dependency
- `flint pkg remove <name>` — remove dependency
- `flint pkg install` — install all dependencies (lock file enforced)
- `flint pkg publish` — publish to official registry
- `flint pkg search <query>` — search registry
- Registry format: simple HTTPS-based Git-backed package index
- Dependency version locking via `flint.lock`
- Workspace support: manage multiple packages under one root

### Testing Framework (`stdlib/test/`):
- Built-in `test` blocks at file level
- `assert`, `assert_eq`, `assert_ne`, `assert_throws`
- `flint test` — run all tests
- `flint test --watch` — re-run on file change (hot reload)
- `flint test --coverage` — generate coverage report
- Parallel test execution by default

---

## PHASE 10 — WebAssembly & Frontend

- WASM compilation target via LLVM (`--target wasm32`)
- Reactive UI system (`stdlib/ui/`)
  - Component-based architecture (similar to React but Flint-native)
  - Declarative syntax using Flint's block syntax
  - State management built-in (`state { }` blocks)
  - CSS-in-Flint (`style { }` blocks)
- Fullstack support: share types/validation between backend & frontend
- Cross-platform desktop UI via WebView bindings

```flint
component Counter:
  state count = 0

  view:
    div:
      h1 "Count: ${count}"
      button("Increment") onClick: count += 1
```

---

## PHASE 11 — Security

- **Memory safety**: ownership model (Rust-inspired borrow checker, simplified for UX)
- **Null safety**: compiler rejects unguarded nullable access at compile time
- **Sandboxed execution**: `sandbox { }` blocks restrict FS/network access
- **Permission system**: `@require(fs.read, net.outbound)` capability annotations
- **Safe concurrency**: data-race prevention at compile time
- **Secure package verification**: SHA256 + GPG signature on all packages
- **Encrypted env vars**: `@secret` annotation encrypts env values at rest
- **Safer pointer handling**: no raw pointer arithmetic; explicit `unsafe { }` blocks required

---

## PHASE 12 — Big Project & Enterprise Features

- **Monorepo support**: `flint.workspace.toml` defines workspace members
- **Dependency injection**: `@inject`, `@provide`, `@singleton` annotations
- **Modular architecture**: explicit `mod`, `pub`, `priv`, `export` visibility
- **Scalable runtime**: auto-tune thread pool + GC based on available hardware
- **Docker integration**: `flint deploy docker` generates optimized Dockerfile
- **CI/CD**: `flint ci init` scaffolds GitHub Actions / GitLab CI configs
- **Cloud deployment**: `flint deploy --target gcp|aws|fly` adapters
- **Logging**: structured JSON log output, configurable levels, trace IDs
- **Metrics**: Prometheus-compatible `/metrics` endpoint via `@metrics` annotation
- **Distributed tracing**: OpenTelemetry integration in HTTP + async runtime

---

## PHASE 13 — VSCode Extension

In `vscode-extension/`:

- Use the generated LSP server as the backend
- Register `.flint` file extension with syntax grammar (TextMate grammar file)
- Syntax highlighting for all Flint tokens
- Bracket matching, auto-close, auto-indent
- Snippets for common patterns (`fn`, `struct`, `match`, `async fn`, `test`, etc.)
- Commands: Format Document, Run File, Open REPL, Run Tests
- Status bar: current Flint version, active file diagnostics count
- Publish to VSCode Marketplace

---

## ERROR MESSAGE PHILOSOPHY

All compiler + runtime errors must follow this format:

```
error[E042]: type mismatch
  --> src/main.flint:14:8
   |
14 |   let x: Int = "hello"
   |                ^^^^^^^ expected `Int`, found `Str`
   |
help: did you mean to declare x as `Str`?
   |
14 |   let x: Str = "hello"
```

- Always include file path, line, column
- Show the relevant source line with a caret `^` underline
- Use plain English, not compiler jargon
- Always include a `help:` suggestion where possible
- Group related errors to avoid noise spam
- Color output: red for errors, yellow for warnings, blue for notes

---

## SYNTAX DESIGN REFERENCE

Here is a representative Flint program covering most features.
Use this as the canonical syntax reference throughout implementation:

```flint
# flint example — canonical syntax reference

import http from "flint:http"
import { User, Post } from "./models"

# Constants
const VERSION: Str = "1.0.0"

# Generic function with default arg
fn repeat<T>(value: T, times: Int = 3) -> [T]:
  return [value] * times

# Struct with methods
struct Point:
  x: Float
  y: Float

  fn distance(self, other: Point) -> Float:
    return ((self.x - other.x)**2 + (self.y - other.y)**2)**0.5

# Enum with associated values
enum Result<T, E>:
  Ok(value: T)
  Err(error: E)

# Interface
interface Serializable:
  fn toJson(self) -> Str
  fn fromJson(data: Str) -> Self

# Async function
async fn loadUser(id: Int) -> User | Null:
  let resp = await http.get("/users/${id}")
  if resp.status == 200:
    return resp.json<User>()
  return null

# Pattern matching
fn describeShape(s: Shape) -> Str:
  match s:
    Circle(r)         => "circle with radius ${r}"
    Rect(w, h)        => "rect ${w}x${h}"
    _                 => "unknown shape"

# Lambda + method chaining
let names = users
  .filter(u => u.active and u.age >= 18)
  .map(u => u.name.trim())
  .sort()

# Destructuring
let { name, email, ...rest } = currentUser
let [head, ...tail] = queue

# Concurrency
async fn fetchAll(ids: [Int]) -> [User]:
  let tasks = ids.map(id => spawn loadUser(id))
  return await Promise.all(tasks)

# AI block
async fn summarize(text: Str) -> Str:
  return await prompt("Summarize in 2 sentences: ${text}")

# Test block
test "repeat function":
  assert_eq(repeat(1, 3), [1, 1, 1])
  assert_eq(repeat("hi", 2), ["hi", "hi"])
```

---

## IMPLEMENTATION ORDER (strict)

Follow this exact sequence. Do not skip ahead:

1. Repo scaffold + Cargo workspace
2. Lexer + token tests
3. AST definitions
4. Parser + round-trip printer
5. Type checker (basic types first, then generics, then async)
6. Bytecode compiler + VM
7. REPL (minimal, using VM)
8. Standard library core (`fs`, `json`, `http` client)
9. LLVM native backend
10. Package manager + project scaffold
11. Formatter + linter
12. LSP server
13. AI stdlib
14. Full backend stdlib (db, orm, auth, ws, jobs)
15. WASM + frontend stdlib
16. Security (borrow checker simplified, sandbox, permissions)
17. Enterprise features (monorepo, DI, Docker, CI/CD)
18. VSCode extension

---

## QUALITY REQUIREMENTS

- Every phase must have unit tests before moving to the next
- The REPL must work end-to-end by Phase 7 at the latest
- All error messages must follow the format specified above — no raw panics
- CI must run: `cargo test`, `flint test`, `flint lint`, `flint fmt --check`
- The compiler must handle the canonical syntax example in this prompt without errors by Phase 6
- Benchmarks: compile hello-world in < 200ms, start HTTP server in < 50ms

---

## DELIVERABLES

At completion, the following commands must work:

```sh
flint new my-app          # scaffold project
cd my-app
flint run                 # run main.flint
flint build               # native binary
flint build --target wasm # WebAssembly
flint test                # run tests
flint fmt                 # format code
flint lint                # lint code
flint repl                # interactive REPL
flint lsp                 # start language server
flint pkg add lodash      # add package
flint pkg install         # install deps
flint deploy docker       # generate Dockerfile
```

Begin with Phase 0. Confirm the repo structure and Cargo workspace compile cleanly,
then proceed to Phase 1.

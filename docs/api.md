# Flint API Reference

## Standard Library

### std::http

HTTP client and server.

```flint
import http from "flint:http"

# Client
let client = http.client()
let response = client.get("https://api.example.com")
let json_data = response.json()

# Server
let app = http.server()
app.get("/users/:id", async fn(req, res):
  res.json({ id: 1, name: "Alice" })
)
app.listen(3000)
```

**Functions:**
- `client()` - Create HTTP client
- `server()` - Create server builder

### std::json

JSON parsing and serialization.

```flint
import json from "flint:json"

let data = json.parse('{"name": "Flint"}')
let str = json.stringify(data)
let pretty = json.pretty(data)
```

**Functions:**
- `parse(str)` - Parse JSON string
- `stringify(value)` - Convert to JSON string
- `pretty(value)` - Pretty-print JSON

### std::fs

File system operations.

```flint
import fs from "flint:fs"

let content = fs.read("file.txt")
fs.write("output.txt", "content")
let exists = fs.exists("file.txt")
let files = fs.list_dir(".")
```

**Functions:**
- `read(path)` - Read file contents
- `write(path, content)` - Write file
- `exists(path)` - Check if file exists
- `list_dir(path)` - List directory contents

### std::env

Environment variables.

```flint
import env from "flint:env"

let value = env.get("API_KEY")
env.set("DEBUG", "true")
env.remove("TEMP")
```

**Functions:**
- `get(key)` - Get environment variable
- `set(key, value)` - Set environment variable
- `remove(key)` - Remove environment variable

### std::ai

AI integration.

```flint
import ai from "flint:ai"

# Configure
ai.config(model: "claude-3", api_key: "...")

# Prompt
let result = await ai.prompt("Hello, world!")

# Agent
agent my_agent:
  tools: [web_search]
  
  async fn search(query: Str) -> Str:
    return await self.web_search(query)
```

**Functions:**
- `prompt(text)` - Execute prompt
- `complete(text)` - Complete text
- `embed(text)` - Generate embeddings
- `config(options)` - Configure AI client

### std::log

Structured logging.

```flint
import log from "flint:log"

log.info("Application started")
log.warn("Deprecated API used")
log.error("Connection failed")
log.debug("Debug info")
```

**Functions:**
- `info(msg)` - Info level log
- `warn(msg)` - Warning level log
- `error(msg)` - Error level log
- `debug(msg)` - Debug level log

### std::test

Testing framework.

```flint
import test from "flint:test"

test.assert_eq(1, 1)
test.assert_true(condition)
test.assert_throws(|| panic("error"), "error")

test "my test":
  assert_eq(calculate(2, 3), 5)
```

**Functions:**
- `assert(condition)` - Assert true
- `assert_eq(a, b)` - Assert equality
- `assert_ne(a, b)` - Assert inequality
- `assert_throws(f, msg)` - Assert throws

### std::db

Database connections.

```flint
import db from "flint:db"

let conn = db.postgres("postgresql://user:pass@host/db")
let rows = conn.query("SELECT * FROM users")
conn.execute("INSERT INTO users (name) VALUES (?)", ["Alice"])
```

**Functions:**
- `sqlite(path)` - Connect to SQLite
- `postgres(url)` - Connect to PostgreSQL
- `mysql(url)` - Connect to MySQL
- `redis(url)` - Connect to Redis

### std::security

Security utilities.

```flint
import security from "flint:security"

# Encrypted env vars
@secret
let api_key = env("API_KEY")

# Sandbox
let sandbox = security.sandbox()
sandbox.allow_fs(["/tmp"])

# Capabilities
@require(fs.read, net.outbound)
fn access(): ...
```

**Functions:**
- `sandbox()` - Create sandbox
- `encrypt_env(key, value)` - Encrypt value
- `decrypt_env(key, value)` - Decrypt value

## Runtime API

### runtime::async

Async runtime.

```flint
import runtime from "flint:runtime"

# Sleep
await runtime.sleep(1.0)

# Spawn task
let handle = runtime.spawn(async_task())

# Join handles
await runtime.join_all(handles)
```

### runtime::channel

Message passing.

```flint
import channel from "flint:channel"

let (tx, rx) = channel<Int>()
tx.send(42)
let value = rx.recv()
```

### runtime::task

Task management.

```flint
import task from "flint:task"

let id = task.id()
let status = task.status()
task.cancel()
```

## Compiler API

### compiler::lexer

Tokenization.

```flint
import lexer from "flint:compiler:lexer"

let tokens = lexer.tokenize(source)
for token in tokens:
  print(token)
```

### compiler::parser

AST generation.

```flint
import parser from "flint:compiler:parser"

let ast = parser.parse(source)
```

### compiler::typechecker

Type checking.

```flint
import typechecker from "flint:compiler:typechecker"

let errors = typechecker.check(ast)
```

### compiler::codegen

Code generation.

```flint
import codegen from "flint:compiler:codegen"

# LLVM IR
let ir = codegen.generate_llvm_ir(ast)

# WASM
let wasm = codegen.compile_to_wasm(ast)
```

## VM API

### vm

Virtual machine execution.

```flint
import vm from "flint:vm"

let bytecode = vm.compile(ast)
let result = vm.run(bytecode)
```
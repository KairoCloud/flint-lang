# Flint Language Specification

## Introduction

Flint is a statically typed, compiled language with type inference, inspired by Rust, Python, and Swift.

## Lexical Structure

### Keywords

```
let, var, const, fn, return, if, else, elif, for, while, match,
in, is, not, and, or, true, false, null,
type, interface, trait, impl, enum, struct,
import, export, from, as,
async, await, spawn, channel, send, recv,
pub, priv, static, mut, self, super,
use, mod, test, assert, ai, prompt, agent, extend, where
```

### Literals

- **Integers**: `42`, `0xFF`, `0b1010`
- **Floats**: `3.14`, `1.5e-10`
- **Booleans**: `true`, `false`
- **Strings**: `"hello"`, `'world'`, `"""multiline"""`

### Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `**` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical | `and`, `or`, `not` |
| Bitwise | `&`, `|`, `^`, `<<`, `>>` |
| Range | `..`, `...` |
| Other | `->`, `=>`, `?`, `??`, `?.` |

## Types

### Primitive Types

| Type | Description |
|------|-------------|
| `Int` | 64-bit signed integer |
| `Float` | 64-bit floating point |
| `Bool` | Boolean (`true`/`false`) |
| `Str` | Unicode string |
| `Char` | Single Unicode character |
| `Null` | Null value |

### Composite Types

```flint
# Array
let arr: [Int] = [1, 2, 3]

# Tuple
let tuple: (Int, Str) = (42, "hello")

# Map
let map: {Str: Int} = {"a": 1, "b": 2}

# Set
let set: {Int} = {1, 2, 3}

# Generic
let list: List<Int> = [1, 2, 3]
let opt: Option<Int> = Some(42)
let result: Result<Int, Str> = Ok(42)
```

### Union Types

```flint
# Nullable
let x: ?Int = 42

# Union
let y: Int | Str = "hello"

# Pattern matching on unions
match value:
  Int => "integer"
  Str => "string"
  Null => "null"
```

## Expressions

### Variables

```flint
let x = 42           # Immutable
var y = 0            # Mutable
const Z = 100        # Compile-time constant
```

### Functions

```flint
# Basic function
fn add(a: Int, b: Int) -> Int:
  return a + b

# With default arguments
fn greet(name: Str, greeting: Str = "Hello") -> Str:
  return "${greeting}, ${name}!"

# Generic function
fn map<T, U>(list: [T], f: (T) -> U) -> [U]:
  ...

# Lambda
let double = (x: Int) => x * 2

# Method
fn Point.distance(self, other: Point) -> Float:
  let dx = self.x - other.x
  let dy = self.y - other.y
  return (dx * dx + dy * dy) ** 0.5
```

### Control Flow

```flint
# If expression
let x = if condition { 1 } else { 0 }

# Match expression
let desc = match n:
  0 => "zero"
  1 => "one"
  _ => "many"

# For loop
for item in collection:
  print(item)

# While loop
while count > 0:
  count -= 1
```

## Statements

### Declarations

```flint
# Variable
let name = "Flint"
var count: Int = 0

# Function
fn foo(): ...

# Struct
struct Point:
  x: Float
  y: Float

# Enum
enum Color:
  Red
  Green
  Blue
  Custom(r: Int, g: Int, b: Int)

# Interface
interface Drawable:
  fn draw(self) -> Void

# Trait
trait Serializable:
  fn serialize(self) -> Str

# Impl
impl Point:
  fn distance(self, other: Point) -> Float: ...
```

### Control

```flint
return value
break
continue
```

## Concurrency

```flint
# Async function
async fn fetch(url: Str) -> Response:
  let resp = await http.get(url)
  return resp.json()

# Channel
let ch = channel<Int>()
spawn:
  send(ch, 42)
let value = recv(ch)

# Parallel
parallel:
  let a = await taskA()
  let b = await taskB()
```

## AI Features

```flint
# Prompt execution
let result = await prompt("Summarize: ${text}")

# AI agent
agent Assistant:
  tools: [web_search, file_read]
  
  async fn answer(question: Str) -> Str:
    let info = await self.web_search(question)
    return summarize(info)
```

## Modules

```flint
# Import
import http from "flint:http"
import { User, Post } from "./models"

# Export
export User, Post, createUser
```

## Attributes

```flint
@require(fs.read, net.outbound)
fn access_resources(): ...

@deprecated
fn old_function(): ...

@ai.complete
fn generate_bio(name: Str) -> Str: ...
```

## Error Handling

```flint
# Try-catch
try:
  risky_operation()
catch e:
  handle_error(e)

# Option handling
let value = maybe_null ?? "default"
let cleaned = input?.trim() ?? ""
```

## Syntax Summary

| Feature | Syntax |
|----------|--------|
| Function | `fn name(params) -> ReturnType: body` |
| Method | `fn name(self, params): body` |
| Lambda | `(params) => expression` |
| Struct | `struct Name: fields` |
| Enum | `enum Name: variants` |
| Interface | `interface Name: methods` |
| Match | `match expr: pattern => result` |
| Async | `async fn name(): body` |
| Channel | `channel<T>()` |
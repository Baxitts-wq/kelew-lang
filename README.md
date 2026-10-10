<p align="center">
  <img src="assets/logo.png" alt="Bexy Systems Logo" width="180" />
</p>

<h1 align="center">Kelew Language Engine</h1>

<p align="center">
  <strong>Stack-based bytecode virtual machine, native compiler, and zero-dependency intelligence runtime engineered by Bexy.</strong>
</p>

<p align="center">
  <a href="#architecture">Architecture</a> &bull;
  <a href="#language-specification">Language Specification</a> &bull;
  <a href="#first-class-functions">Functions</a> &bull;
  <a href="#intelligence-and-standard-library">Intelligence & StdLib</a> &bull;
  <a href="#standalone-binary-bundler">Native Bundler</a> &bull;
  <a href="#embedding-api">Embedding API</a> &bull;
  <a href="#benchmarking-and-tests">Tests</a>
</p>

---

## Overview

Kelew is a compiled, stack-oriented programming language and virtual machine implemented in pure Rust with zero external dependencies. It combines lexical analysis with line/column tracking, recursive-descent parsing into a strongly typed AST, bytecode compilation with constant deduplication and relative jump patching, and execution on a register-free stack runtime.

Kelew is engineered for systems automation, embeddable application scripting, and sovereign standalone distribution.

## Architecture

```
  +-----------------------------------------------------------+
  |                    Source Code (.klw)                     |
  +-----------------------------------------------------------+
                                |
                         [1. Lexer (Scanner)]
                                |
                                v
  +-----------------------------------------------------------+
  |              Token Stream (Span: Line, Col)               |
  +-----------------------------------------------------------+
                                |
                    [2. Recursive-Descent Parser]
                                |
                                v
  +-----------------------------------------------------------+
  |                Abstract Syntax Tree (AST)                 |
  +-----------------------------------------------------------+
                                |
                     [3. Bytecode Compiler]
                                |
                                v
  +-----------------------------------------------------------+
  |              Bytecode Chunk (Opcodes & Pool)              |
  +-----------------------------------------------------------+
                                |
                +---------------+---------------+
                |                               |
                v                               v
    [4. Stack-Based VM Engine]       [5. Native PE Bundler]
                |                               |
                v                               v
         Runtime Execution           Standalone Executable (.exe)
```

## Language Specification

### Keywords and Grammar

| Keyword | Semantic Equivalent | Description |
| :--- | :--- | :--- |
| `T3boun dymek` | `program_start` | Block initialization delimiter |
| `Fio Del puta` | `program_end` | Execution termination delimiter |
| `Jibou` / `import` | `import` | Multi-file module import statement |
| `Kelew` | `let` / `var` | Variable declaration and binding |
| `khedma` | `fn` / `function` | First-class function declaration |
| `raja3` | `return` | Function return statement |
| `Ejacule` | `print` | Standard output writer (`stdout`) |
| `Jib` / `Chouf` | `input` | Standard input reader (`stdin`) |
| `Zaama` | `if` | Conditional branch entry |
| `Walou` | `else` | Alternative conditional branch |
| `Khalas` / `Baraka` | `endif` / `end` | Block terminator (functions, conditions, loops) |
| `Dima` | `while` | Loop entry |
| `Sah` | `true` | Boolean true |
| `K7hba` | `false` | Boolean false |

### First-Class Functions

Kelew supports function declarations with lexical parameter binding, recursion, and return values:

```kelew
T3boun dymek

# Recursive factorial function
khedma factorielle(n)
    Zaama n <= 1
        raja3 1
    Baraka
    raja3 n * factorielle(n - 1)
Baraka

Kelew resultat = factorielle(5)
Ejacule resultat   # Output: 120

Fio Del puta
```

### Data Structures (Lists & Dictionaries)

Kelew provides native dynamic lists and associative maps with in-place mutation and subscript indexing:

```kelew
T3boun dymek

# Dynamic lists
Kelew serveurs = ["pc1-stream", "pc2-encoder"]
serveurs[1] = "pc2-primary"
Ajouter(serveurs, "pc3-backup")
Ejacule serveurs[1]           # Output: pc2-primary
Ejacule Longueur(serveurs)    # Output: 3

# Dictionaries / HashMaps
Kelew cluster = {
    "nom": "Bexy Cluster",
    "nodes": 3,
    "actif": Sah
}
cluster["statut"] = "OPTIMAL"
Ejacule cluster["statut"]     # Output: OPTIMAL

Fio Del puta
```

### Multi-File Module System (`Jibou`)

Kelew supports modular multi-file architectures using the `Jibou "path.klw"` statement:

- **Relative Path Resolution**: Paths are resolved relative to the importing file's directory.
- **Single-Load Deduplication**: Modules are loaded and evaluated at most once across the entire dependency graph (diamond dependencies execute once).
- **Circular Dependency Detection**: Import cycles are detected at compile-time and rejected with explicit error diagnostics.
- **Monolithic Bytecode Inlining**: All imported modules are compiled directly into the root chunk, ensuring that `kelew build` produces a 100% self-contained standalone `.exe` with zero external file dependencies at runtime.

Example multi-file program:

```kelew
# examples/modules/maths.klw
khedma carre(n)
    raja3 n * n
Baraka
```

```kelew
# examples/modules/main.klw
T3boun dymek
Jibou "maths.klw"

Kelew res = carre(5)
Ejacule res   # Output: 25

Fio Del puta
```

### Visual Compiler & Runtime Diagnostics

Kelew features an integrated visual diagnostic reporting engine providing precise file, line, and column tracking with source extracts and caret indicators:

```text
[Erreur Compilation] Module introuvable a la ligne 2: 'inexistant.klw' dans main.klw:2:1
  |
  2 | Jibou "inexistant.klw"
    | ^--- Module introuvable a la ligne 2: 'inexistant.klw'
  |
```

### Multi-File Runtime Error Resolution & Call Stack Trace

Each bytecode opcode embeds both its line and its source file identifier (`file_id`). When a runtime exception occurs across imported modules or nested function invocations, the virtual machine reconstructs the full hierarchical call stack:

```text
[Erreur Runtime VM]
  Message: Soustraction requiert deux nombres: 'texte' et '5'
  Emplacement: maths.klw:3
  Trace de pile (Call Stack Trace):
    --> maths.klw:3 dans crash_deep()
    --> algebra.klw:8 dans compute()
    --> main.klw:12 dans <script>()
[Erreur Runtime] Soustraction requiert deux nombres: 'texte' et '5' dans maths.klw:3:1
  |
  3 | Kelew invalid = "texte" - 5
    | ^--- Soustraction requiert deux nombres: 'texte' et '5'
  |
```

## Intelligence and Standard Library

Kelew provides a standard library built without external crates:

### Collections and Data Structures

| Function | Signature | Description |
| :--- | :--- | :--- |
| `Ajouter` / `Push` | `Ajouter(liste, element)` | Appends element to list |
| `SupprimerIndex` / `RemoveAt` | `SupprimerIndex(liste, index)` | Removes and returns element at specified index |
| `Cles` / `Keys` | `Cles(dictionnaire)` | Returns list of all keys in dictionary |
| `Valeurs` / `Values` | `Valeurs(dictionnaire)` | Returns list of all values in dictionary |
| `Longueur` / `Len` | `Longueur(collection)` | Returns count of items (strings, lists, maps) |

### AI and Data Extraction

| Function | Signature | Description |
| :--- | :--- | :--- |
| `AskAi` | `AskAi(prompt)` | Queries LLM inference via native transport (uses `GEMINI_API_KEY` with deterministic offline fallback) |
| `JsonGet` | `JsonGet(json_str, key)` | Zero-dependency high-throughput JSON key extractor |

### System and File Operations

| Function | Signature | Description |
| :--- | :--- | :--- |
| `LireFichier` | `LireFichier(path)` | Reads complete file content as string |
| `EcrireFichier` | `EcrireFichier(path, content)` | Writes string data to disk |
| `FichierExiste` | `FichierExiste(path)` | Returns `Sah` (true) if file exists, else `K7hba` |
| `SupprimerFichier` | `SupprimerFichier(path)` | Deletes target file from disk |
| `HttpGet` | `HttpGet(url)` | Performs HTTP/HTTPS GET request |
| `HttpPost` | `HttpPost(url, body)` | Performs HTTP/HTTPS POST request |
| `TempsMs` | `TempsMs()` | Returns current Unix epoch timestamp in milliseconds |
| `Dormir` | `Dormir(ms)` | Suspends thread execution for specified milliseconds |
| `LireEnv` | `LireEnv(key)` | Reads system environment variable |

### AI Workflow Example

```kelew
T3boun dymek

# 1. Parse JSON payload without external dependencies
Kelew payload = "{\"status\": \"active\", \"version\": 1.0, \"cluster\": \"bexy-node-1\"}"
Kelew cluster = JsonGet(payload, "cluster")
Ejacule cluster

# 2. Query intelligence engine
Kelew prompt = "Analyser les metriques de performance du cluster"
Kelew analyse = AskAi(prompt)
Ejacule analyse

Fio Del puta
```

## Virtual Machine Instruction Set

The bytecode compiler emits fixed-width opcodes stored in contiguous chunks:

- `OP_CONSTANT <idx>`: Push value from constant pool to evaluation stack.
- `OP_DEFINE_GLOBAL <idx>`: Bind identifier from constant pool to global register.
- `OP_GET_GLOBAL <idx>`: Load identifier value onto evaluation stack.
- `OP_SET_GLOBAL <idx>`: Mutate global identifier with stack top value.
- `OP_ADD`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_MODULO`: Binary arithmetic evaluation.
- `OP_EQUAL`, `OP_NOT_EQUAL`, `OP_LESS`, `OP_LESS_EQUAL`, `OP_GREATER`, `OP_GREATER_EQUAL`: Binary relational evaluation.
- `OP_JUMP_IF_FALSE <offset>`: Conditional forward jump if stack top is falsy.
- `OP_JUMP <offset>`: Unconditional forward jump.
- `OP_LOOP <offset>`: Unconditional backward jump for loop iterations.
- `OP_CALL <name_idx> <argc>`: Execute user-defined function or native standard library primitive.
- `OP_RETURN`: Return from function scope, preserving evaluated return value.
- `OP_BUILD_LIST <count>`: Construct dynamic list from top stack elements.
- `OP_BUILD_MAP <count>`: Construct associative map from key-value pairs on stack.
- `OP_GET_INDEX`: Read element from list, map, or string by index.
- `OP_SET_INDEX`: Mutate element in list or map in-place.
- `OP_PRINT`: Pop stack top and write to standard output.
- `OP_INPUT <idx>`: Read input from standard input and assign to global variable.
- `OP_HALT`: Terminate virtual machine evaluation.

## Standalone Binary Bundler

Kelew compiles any `.klw` source into a self-executing standalone native executable (`.exe` on Windows):

```bash
cargo run --release -- build examples/intelligent_ai.klw -o app.exe
```

The resulting `app.exe` embeds the compiled bytecode directly within the binary trailer and executes in under 1ms with zero runtime dependencies.

```bash
./app.exe
```

## Embedding API

Kelew provides an embeddable Rust interface (`kelew::Engine`):

```rust
use kelew::Engine;

fn main() {
    let mut engine = Engine::new();
    
    let code = r#"
        T3boun dymek
        khedma calcul(x)
            raja3 x * 2 + 10
        Baraka
        Kelew res = calcul(16)
        Fio Del puta
    "#;
    
    engine.eval(code).expect("Execution failed");
    
    if let Some(val) = engine.get_global("res") {
        println!("Value computed by Kelew VM: {}", val);
    }
}
```

## CLI Usage

### Running a Script

```bash
cargo run --release -- run examples/intelligent_ai.klw
```

With real-time stack tracing:

```bash
cargo run --release -- run examples/intelligent_ai.klw --trace
```

### Bytecode Disassembly

```bash
cargo run --release -- disasm examples/intelligent_ai.klw
```

### Static Analysis

```bash
cargo run --release -- check examples/intelligent_ai.klw
```

### Interactive REPL

```bash
cargo run --release -- repl
```

## Benchmarking and Tests

Run the test suite:

```bash
cargo test
```

## Organization

Engineered and maintained by **Bexy Systems**.

## License

MIT License.

# Kelew

Kelew is a stack-based bytecode virtual machine and compiler implemented in pure Rust with zero external dependencies.

It features a multi-pass pipeline: lexical analysis with source location tracking, recursive-descent parsing into a typed abstract syntax tree (AST), bytecode compilation with constant deduplication and relative jump patching, and execution on a stack machine runtime.

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
    [4. Stack-Based VM Engine]       [5. Disassembler Engine]
                |
                v
         Runtime Execution
```

## Language Specification

### Keywords and Grammar

| Keyword | Semantic Equivalent | Description |
| :--- | :--- | :--- |
| `T3boun dymek` | `program_start` | Block initialization delimiter |
| `Fio Del puta` | `program_end` | Execution termination delimiter |
| `Kelew` | `let` / `var` | Variable declaration and reassignment |
| `Ejacule` | `print` | Standard output writer (`stdout`) |
| `Jib` / `Chouf` | `input` | Interactive standard input reader (`stdin`) |
| `Zaama` | `if` | Conditional branch entry |
| `Walou` | `else` | Alternative branch |
| `Khalas` | `endif` | Conditional branch terminator |
| `Dima` | `while` | Loop entry |
| `Baraka` | `endwhile` | Loop terminator |
| `Sah` | `true` | Boolean true |
| `K7hba` | `false` | Boolean false |

### Operators

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Relational: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Grouping: `(`, `)`

## Virtual Machine Instruction Set

The bytecode compiler emits fixed-width instructions stored inside contiguous chunks:

- `OP_CONSTANT <idx>`: Push value from constant pool to evaluation stack.
- `OP_DEFINE_GLOBAL <idx>`: Bind identifier from constant pool to global register.
- `OP_GET_GLOBAL <idx>`: Load identifier value onto evaluation stack.
- `OP_SET_GLOBAL <idx>`: Mutate global identifier with stack top value.
- `OP_ADD`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_MODULO`: Binary arithmetic evaluation.
- `OP_EQUAL`, `OP_NOT_EQUAL`, `OP_LESS`, `OP_LESS_EQUAL`, `OP_GREATER`, `OP_GREATER_EQUAL`: Binary relational evaluation.
- `OP_JUMP_IF_FALSE <offset>`: Conditional forward jump if stack top is falsy.
- `OP_JUMP <offset>`: Unconditional forward jump.
- `OP_LOOP <offset>`: Unconditional backward jump for loop iterations.
- `OP_PRINT`: Pop stack top and write to standard output.
- `OP_INPUT <idx>`: Read input from standard input and assign to global variable.
- `OP_HALT`: Stop virtual machine evaluation.

## CLI Usage

### Running a program

```bash
cargo run --release -- run examples/benchmark.klw
```

To run with real-time stack tracing:

```bash
cargo run --release -- run examples/benchmark.klw --trace
```

### Disassembling bytecode

The disassembler outputs hexadecimal instruction offsets, line mappings, opcode mnemonics, and resolved operands:

```bash
cargo run --release -- disasm examples/benchmark.klw
```

Example output:

```text
┌──────────────────────────────────────────────────────────────┐
│  KELEW BYTECODE DISASSEMBLY : examples/benchmark.klw         │
└──────────────────────────────────────────────────────────────┘
OFFSET LIGNE  OPCODE             OPERANDES / DETAILS     
------ ------ ------------------ ------------------------
0000     9    OP_CONSTANT        #0    (100)
0001     |    OP_DEFINE_GLOBAL   #1    (base)
0002    10    OP_CONSTANT        #2    (5)
0003     |    OP_DEFINE_GLOBAL   #3    (multiplicateur)
0004    11    OP_CONSTANT        #4    (2)
0005     |    OP_DEFINE_GLOBAL   #5    (dividende)
0006    12    OP_GET_GLOBAL      #1    (base)
0007     |    OP_GET_GLOBAL      #3    (multiplicateur)
0008     |    OP_MULTIPLY        (*)
0009     |    OP_GET_GLOBAL      #5    (dividende)
000A     |    OP_DIVIDE          (/)
000B     |    OP_DEFINE_GLOBAL   #6    (score)
0012    19    OP_GET_GLOBAL      #6    (score)
0013     |    OP_CONSTANT        #9    (200)
0014     |    OP_GREATER_EQUAL   (>=)
0015     |    OP_JUMP_IF_FALSE   -> 0019
0027    34    OP_GET_GLOBAL      #15   (compteur)
0029     |    OP_GREATER         (>)
002A     |    OP_JUMP_IF_FALSE   -> 0034
0033    34    OP_LOOP            -> 0027 (retour)
0036     0    OP_HALT            (Baraka/FioDelPuta)
```

### Static Analysis

Validates token stream, AST derivation, and bytecode compilation without running the program:

```bash
cargo run --release -- check examples/benchmark.klw
```

### Interactive REPL

```bash
cargo run --release -- repl
```

## Testing

```bash
cargo test
```

## License

MIT License.

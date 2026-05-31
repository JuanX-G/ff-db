# ff-db

A lightweight, zero-dependency, flat-file relational database engine written from scratch in Rust. This project features a hand-written lexical analyzer, an SQL query parser, an AST evaluation engine, and a custom disk storage layout.

> **Note:** This project is designed as an embedded, low-overhead database for simple data storage requirements and custom SQL engine implementation reference.

---

## Features

* **Custom Storage Engine:** Manages physical database tables via standard file I/O using a plain-text, comma-separated format.
* **Hand-Written SQL Pipeline:**
  * **Lexer / Tokenizer:** Processes raw string queries into structured `SqlToken` streams.
  * **Parser:** Generates an Abstract Syntax Tree (AST) supporting core query syntax.
  * **Query Engine:** Walks the AST to run filtering logic against disk rows.
* **Strict Type Checking:** Validates structural modifications and entries against native schemas at runtime (`INT` and `TEXT`).
* **Robust Custom Error Subsystem:** Explicit diagnostics for column misses, syntax mismatches, file I/O errors, and type casting exceptions.

---

## System Architecture

Your SQL commands move through a classic data pipeline before modifying state on disk:

1. **Lexical Analysis (`Lexer`)**: Converts raw string queries (e.g., `"SELECT name FROM test_table"`) into structural token streams.
2. **Syntactic Analysis (`Parser`)**: Constructs an Abstract Syntax Tree wrapped inside a secure `ASTRootWrapper`.
3. **Evaluation Engine (`Engine`)**: Inspects runtime database fields, confirms type matches, evaluates conditional filtering logic (`WHERE`), and executes operations against active tables.

---
## SQL Language Support Matrix

| Clause / Keyword | Target / Operator | Status | Description |
| :--- | :--- | :--- | :--- |
| `SELECT` | Column Projections |  Supported | Extracts specific columns from records |
| `INSERT INTO` | Value Insertion |  Supported | Appends rows with automatic default fill-ins |
| `WHERE` | Filtering Logic |  Supported | Conditionally filters target datasets |
| `AND` | Logical Intersection |  Supported | Combines multiple condition states together |
| `OR`, `LIKE` | Advanced Matchers | ⏳ Planned | Extended conditional matching capabilities |
| `=`, `!=` | Equality Filters |  Supported | Text and Integer equivalence operations |
| `<`, `>` | Range Boundaries |  Supported | Strict value comparisons bounds for `INT` types |

## Storage Format

Tables are stored inside plain-text flat files using a dedicated header syntax declaring column names and primitive types:

```text
id: INT, name: TEXT
0, Bob
1, Alice
2, Rob
3, Jane
4, Tod
5, Ann

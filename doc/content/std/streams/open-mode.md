---
kind: enum
id: std.streams.OpenMode
title: OpenMode
summary: Maps dynamic stream opening onto PHP-compatible mode semantics.
name: OpenMode
module: streams
typeParameters: []
interfaces: []
constants: []
properties: []
cases:
  - Read
  - Write
  - Append
  - ReadWrite
  - ReadWriteTruncate
  - ReadWriteAppend
  - CreateExclusive
status: experimental
availability: partial
notice:
  The handle model, stream interfaces, factories, modes, and exceptions are a
  design proposal. They are not implemented in this repository and may change as runtime
  integration proceeds.
version: "0.1"
---

| Case                | PHP mode | Requested capabilities             |
| ------------------- | -------- | ---------------------------------- |
| `Read`              | `r`      | Read and seek                      |
| `Write`             | `w`      | Write and seek; truncate or create |
| `Append`            | `a`      | Write and seek; create if missing  |
| `ReadWrite`         | `r+`     | Read, write, and seek              |
| `ReadWriteTruncate` | `w+`     | Read, write, and seek; truncate    |
| `ReadWriteAppend`   | `a+`     | Read, write, and seek; append      |
| `CreateExclusive`   | `x+`     | Read, write, and seek; new target  |

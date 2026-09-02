---
kind: enum
id: std.streams.WriteMode
title: WriteMode
summary: Controls file creation, truncation, and append behavior.
name: WriteMode
module: streams
typeParameters: []
interfaces: []
constants: []
properties: []
cases:
  - OpenExisting
  - Truncate
  - Append
  - Create
  - CreateExclusive
status: experimental
availability: proposed
notice: Writable-file modes are proposed and are not implemented.
version: "0.1"
---

| Case              | Existing path     | Missing path | Initial position |
| ----------------- | ----------------- | ------------ | ---------------- |
| `OpenExisting`    | Preserve contents | Fail         | Start            |
| `Truncate`        | Truncate          | Create       | Start            |
| `Append`          | Preserve contents | Create       | End              |
| `Create`          | Preserve contents | Create       | Start            |
| `CreateExclusive` | Fail              | Create       | Start            |

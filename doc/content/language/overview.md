---
kind: guide
id: guide.languageOverview
title: Overview
summary: Reference for THP syntax, types, declarations, and runtime semantics.
nav:
  section: language
  order: 10
status: experimental
availability: partial
notice: >-
  Some linked chapters describe executable behavior and others are design
  proposals. Use each page's availability badge and the implementation-status
  matrix to distinguish them.
---

This reference follows the broad organization of the
[PHP Language Reference](https://www.php.net/manual/en/langref.php), while
documenting THP's own typed language and standalone runtime. PHP behavior is not
automatically THP behavior.

## Syntax and values

| Topic                                                          | Contents                                      |
| -------------------------------------------------------------- | --------------------------------------------- |
| [Basic syntax](thp:guide.languageBasicSyntax)                  | Source files, tags, statements, and comments. |
| [Types](thp:guide.languageTypes)                               | Scalar, compound, nullable, and union types.  |
| [Variables](thp:guide.languageVariables)                       | Bindings, inference, and explicit types.      |
| [Constants](thp:guide.languageConstants)                       | Named and declaration-scoped constants.       |
| [Expressions](thp:guide.languageExpressions)                   | Values produced by evaluating source code.    |
| [Operators](thp:guide.languageOperators)                       | Arithmetic, comparison, logic, and access.    |
| [Predefined variables](thp:guide.languagePredefinedVariables)  | Runtime-provided bindings.                    |
| [Resources and streams](thp:guide.languageResourcesAndStreams) | Typed native handles, ownership, and I/O.     |

## Flow and declarations

| Topic                                                      | Contents                                   |
| ---------------------------------------------------------- | ------------------------------------------ |
| [Control structures](thp:guide.languageControlStructures)  | Branches, loops, matching, and transfers.  |
| [Functions](thp:guide.languageFunctions)                   | Typed callables, parameters, and returns.  |
| [Classes and objects](thp:guide.languageClassesAndObjects) | Classes, interfaces, and object members.   |
| [Namespaces](thp:guide.languageNamespaces)                 | Qualified names and imports.               |
| [Enumerations](thp:guide.languageEnumerations)             | Closed sets of named cases.                |
| [Generators](thp:guide.languageGenerators)                 | Proposed resumable sequence producers.     |
| [Attributes](thp:guide.languageAttributes)                 | Proposed declaration metadata syntax.      |
| [References](thp:guide.languageReferences)                 | Recognized but incomplete aliasing syntax. |

## Failures and predefined APIs

| Topic                                      | Contents                             |
| ------------------------------------------ | ------------------------------------ |
| [Errors](thp:guide.languageErrors)         | Diagnostics and runtime failures.    |
| [Exceptions](thp:guide.languageExceptions) | Throwing, catching, and propagation. |
| Predefined exceptions                      | Proposed throwable hierarchy.        |
| Predefined interfaces and classes          | Engine-level contracts.              |
| Predefined attributes                      | Built-in declaration metadata.       |

The [implementation-status matrix](thp:guide.implementationStatus) is the
detailed authority for what the current executable accepts. Normative behavior
will eventually live in the language specification.

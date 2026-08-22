---
kind: guide
id: guide.languagePredefinedVariables
title: Predefined variables
summary: Records the status of variables supplied automatically by the THP runtime.
nav:
  section: language
  order: 180
status: experimental
availability: proposed
notice: >-
  THP does not yet define a stable set of runtime-provided variables.
---

PHP supplies superglobals such as `$_SERVER`, `$_GET`, and `$_POST`, plus
command-line bindings such as `$argc` and `$argv`. Their presence, shapes, and
lifecycle are PHP runtime behavior and must not be assumed in THP.

Current THP documentation establishes no stable superglobals or command-line
variables. Web requests, environment access, process arguments, sessions,
cookies, and uploaded files will need typed APIs or explicitly documented
bindings before applications can depend on them.

Ordinary variables continue to use `$name` syntax.

## See also

- [Variables](thp:guide.languageVariables)
- [PHP predefined variables](https://www.php.net/manual/en/reserved.variables.php)

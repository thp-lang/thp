---
kind: guide
id: guide.typeAnalysisHir
title: Type analysis and HIR
summary: See how THP resolves semantic types, validates declarations, and produces typed high-level IR.
nav:
  section: internals
  order: 60
status: experimental
availability: implemented
notice: >-
  This page covers the implemented semantic analyzer; unsupported language
  constructs are rejected before HIR.
---

High-level IR (HIR) is the first representation in which every executable
expression has a semantic type and every callable or local reference has a
compiler identity.

```thp
<?thp
function greet(string $name): string {
    return "Hello, " . $name;
}

echo greet("Ada");
```

The AST contains written names such as `string`, `$name`, and `greet`. HIR
replaces them with `Type::String`, `LocalId(0)`, and `FunctionId(1)`. The call is
bound to parameter zero and its result is known to be a string before lowering
continues.

An abbreviated `thp inspect --emit=hir greet.thp` excerpt is:

```text
Function {
  id: FunctionId(1),
  name: "greet",
  parameters: [LocalId(0)],
  locals: [Local { id: LocalId(0), name: "name", ty: String, ... }],
  return_type: String,
  body: [Return(Some(TypedExpr {
    kind: Binary { op: Concatenate, ... },
    ty: String,
    ...
  }))]
}
```

## What semantic analysis checks

HIR lowering builds the function, class, interface, property, method, and local
tables used by later phases. It validates assignments and returns, operator
operands, calls and argument binding, collection element types, visibility,
inheritance, overrides, trait composition, constructors, exception catches,
and the implemented control-flow requirements.

Diagnostics keep the AST span that caused the mismatch. For example, `echo`
currently requires a string; passing an integer produces a type diagnostic on
the expression rather than a runtime conversion.

If semantic diagnostics exist, the partial HIR remains inspectable but MIR and
bytecode are not produced. HIR never prints its diagnostics and does not know
how the CLI will render them.

## Design choices compared with PHP

PHP keeps an expression's type primarily in its runtime value. Parameter,
property, and return declarations add useful checks, but operators, conditions,
and many calls are designed around dynamic values and coercion. PHP also allows
variance in compatible method overrides.

THP chooses to assign every implemented expression a type in HIR. Conditions
must already be `bool`; equality and arithmetic require accepted operand types;
arguments are bound and checked before execution; and current overrides match
staticness, parameter shape and types, defaults, and return type exactly. This
rejects programs that PHP could evaluate through truthiness or conversion, but
later phases no longer need to rediscover what an operation means for its
operands.

Typed functions next enter [control-flow lowering](thp:guide.controlFlowMir).

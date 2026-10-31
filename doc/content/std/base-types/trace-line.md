---
kind: class
id: std.baseTypes.TraceLine
title: TraceLine
summary: Represents one normalized frame in a captured stack trace.
name: TraceLine
module: base-types
typeParameters: []
interfaces: []
constants: []
properties:
  - name: function
    type: string
    description: "Callable name recorded for this frame. Access: public read-only."
  - name: line
    type: int
    description: "Source or call-site line recorded for this frame. Access: public read-only."
  - name: file
    type: string
    description: "Source file recorded for this frame. Access: public read-only."
  - name: class
    type: string
    description: "Declaring class for a method frame. Access: public read-only."
  - name: object
    type: mixed
    description: "Optional receiver. Construction accepts only an object or null. Access: public read-only."
  - name: type
    type: string
    description: 'Call operator: "->", "::", or "". Access: public read-only.'
  - name: args
    type: mixed
    description: "A vector or null. Captured traces currently store null. Access: public read-only."
status: experimental
availability: implemented
notice: The compiler and reference VM implement this experimental contract.
version: "0.1"
---

This is a final class.

`TraceLine` is an immutable representation of one function, method, or include
frame in a stack trace.

## Construction

| Method                                                      | Description                                                                                                                                       |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`__construct()`](thp:std.baseTypes.TraceLine::__construct) | vector preserves argument order and permits values of any type. A map would incorrectly imply that trace arguments are primarily accessed by key. |

## Behavior

The class is final and immutable. A stack trace is represented as
`vector<TraceLine>`, ordered from the most recent captured frame toward its
callers.
The VM currently records `null` for receiver objects and arguments in captured
frames. `TraceLine` constructors accept a non-negative line, a `type` of `""`,
`"->"`, or `"::"`, an object or `null` receiver, and a vector or `null` for
arguments. Invalid fields throw `InvalidArgumentException`. Property writes
after construction throw `TypeError`.

## Example

```thp
function formatFrame(TraceLine $frame): string {
    $callable = $frame->class . $frame->type . $frame->function;
    return $frame->file . ":" . $frame->line . " " . $callable;
}
```

## See also

- [`Throwable::getTrace()`](thp:std.baseTypes.Throwable)

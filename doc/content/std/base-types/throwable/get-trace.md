---
kind: method
id: std.baseTypes.Throwable::getTrace
title: Throwable::getTrace
summary: Returns structured stack-trace data.
name: getTrace
order: 5
typeParameters: []
parameters: []
returns:
  type: vector<TraceLine>
  description: Returns structured stack-trace data.
errors:
  - description:
      No additional runtime failure beyond parameter validation and failures
      propagated by delegated operations is specified.
related: []
status: experimental
availability: partial
notice:
  This member belongs to an experimental API contract and is not implemented in
  this repository.
version: "0.1"
owner: std.baseTypes.Throwable
visibility: public
modifiers: []
---

[`Throwable`](thp:std.baseTypes.Throwable)`::getTrace()` returns structured stack-trace data.

## Behavior

Returns structured stack-trace data.

This operation does not change receiver state unless the description explicitly states otherwise.

## Example

```thp
$result = $instance->getTrace();
```

The call uses the signature and defaults documented above.

## See also

- [`Throwable`](thp:std.baseTypes.Throwable)

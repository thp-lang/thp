---
kind: class
id: std.spl.NoRewindIterator
title: NoRewindIterator
summary: Shares an inner cursor while suppressing rewind.
name: NoRewindIterator
module: iterators
typeParameters:
  - name: K
    description: Key type preserved from the inner cursor.
  - name: V
    description: Value type preserved from the inner cursor.
parent:
  id: std.spl.IteratorIterator
  arguments:
    - K
    - V
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This experimental no-rewind adapter is proposed and unavailable at runtime.
version: "0.1"
---

`NoRewindIterator<K, V>` extends [`IteratorIterator<K, V>`](thp:std.spl.IteratorIterator)
and preserves its typed keys and values. Construct it with an `Iterator<K, V>`;
aggregates must first supply their cursor. Raw vectors and maps follow the existing
typed iterator conversion rules at the constructor argument boundary.

[`__construct()`](thp:std.spl.NoRewindIterator::__construct) retains the exact
inner cursor without repositioning it. [`rewind()`](thp:std.spl.NoRewindIterator::rewind)
does nothing, including the call made automatically by `foreach`.
[`getInnerIterator()`](thp:std.spl.IteratorIterator::getInnerIterator) returns
that same shared object. The inherited [`valid()`](thp:std.baseTypes.Iterator::valid),
[`key()`](thp:std.baseTypes.Iterator::key), [`value()`](thp:std.baseTypes.Iterator::value),
and [`advance()`](thp:std.baseTypes.Iterator::advance) delegate directly and
propagate inner failures unchanged, including exhausted-access failures.

The adapter does not own a separate position, replay buffer, or snapshot. Partial
traversal starts at the inner cursor's current entry. Other aliases can move or
rewind that same cursor, and the adapter immediately observes the change.
Construction does not initialize an unpositioned source; position it through its
own protocol first when necessary. Exhaustion stays exhausted until an alias
explicitly repositions the inner cursor.

A repeated `foreach` continues at the current entry. `break`, `return`, and a
throw leave the cursor on that entry; `continue` advances it. A loop that runs to
exhaustion leaves a subsequent loop empty. Dropping the adapter releases its
reference to the inner object; other aliases can retain it.

## Proposed example

Unavailable at runtime until this proposal is implemented:

```thp
<?thp
$inner = new VectorIterator([10, 20, 30]);
$inner->advance();
$remaining = new NoRewindIterator($inner);
foreach ($remaining as $value) {
    echo $value; // 20
    break;
}
foreach ($remaining as $value) {
    echo $value; // 20, then 30: break did not advance.
}
```

## See also

- [Iterator rewind contract](thp:std.baseTypes.Iterator::rewind)
- [Iterator catalog](thp:std.iterators)

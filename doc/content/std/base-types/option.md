---
kind: class
id: std.baseTypes.Option
title: Option
summary: Represents either one value or no value.
name: Option
module: base-types
typeParameters:
  - name: T
    description: The type of value a present option contains.
interfaces: []
constants: []
properties: []
status: experimental
availability: proposed
notice: This THP-native value type is proposed and is not yet implemented in this repository.
version: "0.1"
---

This is a final class.

`Option<T>` is a class that makes absence explicit without using a nullable
value. This distinguishes an absent value from a present nullable value when
`T` itself allows `null`.

## Construction

`Option` instances are created through its static factory methods. Its direct
constructor is not public.

### `some()`

```thp
public static function some(T $value): Option<T>
```

Returns an option containing `$value`.

### `none()`

```thp
public static function none(): Option<T>
```

Returns an option representing absence.

## Behavior

`Option::some(null)` and `Option::none()` are distinct when `T` permits `null`.
The class retains its presence state separately from the contained value, so
absence cannot be mistaken for an ordinary value. `Option` is a class rather
than a PHP enum because PHP enum cases cannot carry an associated value.

## Example

```thp
function describe<T>(Option<T> $value): string {
    if ($value->isNone()) {
        return "absent";
    }

    $item = $value->get();
    return "present";
}
```

## See also

- [`Iterator`](thp:std.baseTypes.Iterator)

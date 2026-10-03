--TEST--
collection callback parameter type must match its element type
--FILE--
<?thp
$values = vector_filter([1], fn(string $value): bool => true);
--EXPECTF--
%s048-invalid-collection-callback.phpt:2:26: error[T0005]: expected `string`, found `int`
    2 | $values = vector_filter([1], fn(string $value): bool => true);
      |                          ^

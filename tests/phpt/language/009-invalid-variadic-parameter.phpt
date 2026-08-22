--TEST--
variadic parameters must be final and cannot declare defaults
--FILE--
<?thp

function invalid(int ...$values = [], int $tail): void {
}
--EXPECTF--
%s009-invalid-variadic-parameter.phpt:3:18: error[T0307]: a variadic parameter must be the final parameter
    3 | function invalid(int ...$values = [], int $tail): void {
      |                  ^^^^^^^^^^^^^^^^^^^
%s009-invalid-variadic-parameter.phpt:3:18: error[T0308]: a variadic parameter cannot have a default value
    3 | function invalid(int ...$values = [], int $tail): void {
      |                  ^^^^^^^^^^^^^^^^^^^


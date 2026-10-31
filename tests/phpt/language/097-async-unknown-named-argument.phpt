--TEST--
async rejects an unknown name for its callable argument
--FILE--
<?thp
scheduler_register_default();
async(typo: fn(int $value): int => $value, 1);
--EXPECTF--
%s097-async-unknown-named-argument.phpt:3:%d: error[T0314]: unknown named argument `typo` for `async`
%A097-async-unknown-named-argument.phpt:3:%d: error[T0313]: positional arguments cannot follow named arguments
%A

--TEST--
async checks callable arguments statically
--FILE--
<?thp
scheduler_register_default();
$job = async(fn(int $value): int => $value, "wrong");
--EXPECTF--
%s091-async-callable-type-error.phpt:3:%d: error[T0005]: expected `int`, found `string`
%A

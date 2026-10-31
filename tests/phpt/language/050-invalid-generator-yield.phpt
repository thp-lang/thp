--TEST--
generator yield types and placement are checked statically
--FILE--
<?thp
yield 1;
function bad(): Generator<string, int> {
    yield 2;
    yield "key" => "value";
}
--EXPECTF--
%aerror[T0700]: `yield` is only valid inside a generator function%aerror[T0005]: expected `string`, found `int`%aerror[T0005]: expected `int`, found `string`%a

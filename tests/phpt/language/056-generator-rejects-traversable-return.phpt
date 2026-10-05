--TEST--
generator functions reject Traversable return types during type checking
--FILE--
<?thp
function values(): Traversable<int, int> {
    yield 1;
}
function nullable(): Generator<int, int>|null {
    yield 2;
}
--EXPECTF--
%aerror[T0701]: a generator function must declare `Iterator<K, V>` or `Generator<K, V>`%aerror[T0701]: a generator function must declare `Iterator<K, V>` or `Generator<K, V>`%a

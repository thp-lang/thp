--TEST--
interface implementations are checked after generic substitution
--FILE--
<?thp
interface Value<T> {
    public function value(): T;
}
class Wrong implements Value<int> {
    public function value(): string { return "wrong"; }
}
--EXPECTF--
%s028-generic-interface-implementation.phpt:6:5: error[T0022]: method `Wrong::value` does not satisfy interface `Value`
    6 |     public function value(): string { return "wrong"; }
      |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 %s028-generic-interface-implementation.phpt:3:5: related location
    3 |     public function value(): T;
      |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ required signature is here

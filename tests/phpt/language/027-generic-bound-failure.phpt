--TEST--
explicit generic arguments must satisfy nominal bounds
--FILE--
<?thp
class Entity {}
class Box<T extends Entity> {}
$value = new Box<int>();
--EXPECTF--
%s027-generic-bound-failure.phpt:4:14: error[T1006]: type argument `int` does not satisfy bound `Entity` for `T`
    4 | $value = new Box<int>();
      |              ^^^^^^^^

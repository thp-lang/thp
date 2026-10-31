--TEST--
raw generic names are rejected in static type positions
--FILE--
<?thp
class Box<T> {}
$value: Box = new Box<int>();
--EXPECTF--
%s026-raw-generic-type.phpt:3:9: error[T1002]: type `Box` expects 1 generic arguments, found 0
    3 | $value: Box = new Box<int>();
      |         ^^^

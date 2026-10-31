--TEST--
conflicting constructor inference requests explicit generic arguments
--FILE--
<?thp
class Pair<T> {
    public function __construct(T $left, T $right) {}
}
$pair = new Pair(1, "two");
--EXPECTF--
%s025-ambiguous-generic-inference.phpt:5:9: error[T1011]: constructor arguments infer conflicting generic types
    5 | $pair = new Pair(1, "two");
      |         ^^^^^^^^^^^^^^^^^^
 note: provide explicit generic arguments

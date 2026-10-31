--TEST--
vector_map rejects a callback with no element value
--FILE--
<?thp
vector_map([1], function (int $value): void {});
--EXPECTF--
%s051-void-vector-map-result.phpt:2:17: error[T0302]: `vector_map` callback must return a value
    2 | vector_map([1], function (int $value): void {});
      |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

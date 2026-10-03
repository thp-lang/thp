--TEST--
map_transform rejects a callback with no mapped value
--FILE--
<?thp
map_transform({"a" => 1}, function (int $value, string $key): void {});
--EXPECTF--
%s052-void-map-transform-result.phpt:2:27: error[T0302]: `map_transform` callback must return a value
    2 | map_transform({"a" => 1}, function (int $value, string $key): void {});
      |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

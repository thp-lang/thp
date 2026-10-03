--TEST--
incompatible collection keys cannot be converted to typed iterators
--FILE--
<?thp

$cursor: Iterator<string, int> = [1, 2];
--EXPECTF--
%s039-invalid-iterator-conversion.phpt:3:34: error[T0005]: expected `Iterator<string, int>`, found `vector<int>`
    3 | $cursor: Iterator<string, int> = [1, 2];
      |                                  ^^^^^^

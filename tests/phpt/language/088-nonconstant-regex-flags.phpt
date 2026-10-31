--TEST--
regex result-shaping flags must be compile-time constants
--FILE--
<?thp
$flags: int = 256;
$result = preg_match('/a/', 'a', $flags);
--EXPECTF--
%s088-nonconstant-regex-flags.phpt:3:11: error[T0319]: regex flags must be a non-negative integer constant
    3 | $result = preg_match('/a/', 'a', $flags);
      |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

--TEST--
empty vector slice needs an expected element type
--FILE--
<?thp
$values = vector_slice([], 0);
--EXPECTF--
%s050-empty-slice-needs-type.phpt:2:24: error[T0101]: cannot infer the element type of an empty vector
    2 | $values = vector_slice([], 0);
      |                        ^^

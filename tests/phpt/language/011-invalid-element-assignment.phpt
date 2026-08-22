--TEST--
collection element assignment reports the expected element type
--FILE--
<?thp

$values: vector<int> = [1];
$values[0] = "wrong";
--EXPECTF--
%s011-invalid-element-assignment.phpt:4:14: error[T0005]: expected `int`, found `string`
    4 | $values[0] = "wrong";
      |              ^^^^^^^


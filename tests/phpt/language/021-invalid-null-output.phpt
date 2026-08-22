--TEST--
null requires an explicit output fallback
--FILE--
<?thp

echo null;
$message = "missing: " . null;
--EXPECTF--
%s021-invalid-null-output.phpt:3:6: error[T0002]: `echo` expects `string`, `int`, `float`, or `bool`, got `null`
    3 | echo null;
      |      ^^^^
%s021-invalid-null-output.phpt:4:12: error[T0502]: concatenation accepts only `string`, `int`, `float`, or `bool` values
    4 | $message = "missing: " . null;
      |            ^^^^^^^^^^^^^^^^^^

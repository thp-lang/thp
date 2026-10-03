--TEST--
callable invocation checks argument count at the call site
--FILE--
<?thp
$callback: callable<int, int> = fn(int $value): int => $value;
echo $callback();
--EXPECTF--
%s049-invalid-callable-arity.phpt:3:6: error[T0303]: callable expects 1 arguments, found 0
    3 | echo $callback();
      |      ^^^^^^^^^^^

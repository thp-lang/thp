--TEST--
break outside a loop reports its precise source location
--FILE--
<?thp

break;
--EXPECTF--
%s007-break-outside-loop.phpt:3:1: error[T0620]: `break` is only valid inside a loop
    3 | break;
      | ^^^^^^


--TEST--
Typed file factories reject unsupported stream operations at compile time
--FILE--
<?thp

$stream = Files::openRead("./input.txt");
$stream->writeAll("not writable");
--EXPECTF--
%s:4:10: error[T0404]: method `writeAll` is not defined for `ReadableFileStream`
    4 | $stream->writeAll("not writable");
      |          ^^^^^^^^

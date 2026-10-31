--TEST--
request input remains usable from a coroutine
--FILE--
<?thp
scheduler_register_default();
$reader = async(function (): string {
    $input = Streams::open("thp:/input", OpenMode::Read);
    return $input->read(3);
});
echo await($reader);
--STDIN--
abcdef
--EXPECT--
abc

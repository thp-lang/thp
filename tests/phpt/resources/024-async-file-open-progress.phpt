--TEST--
file opens yield before delivering a stream and preserve open errors
--FILE--
<?thp
scheduler_register_default();
$opener = async(function (): string {
    $stream = Files::openRead("001-memory-binary-roundtrip.phpt");
    return $stream->read(2);
});
$other = async(function (): void { echo "B"; });
echo "A";
echo await($opener);
await($other);
$file = new SplFileObject("001-memory-binary-roundtrip.phpt");
echo $file->fread(2);
$uri = Streams::open("001-memory-binary-roundtrip.phpt", OpenMode::Read);
if ($uri instanceof ReadableStream) { echo $uri->read(2); }
try {
    Files::openRead("definitely-missing-async-open.phpt");
} catch (OpenStreamException $error) {
    echo "E";
}
--EXPECT--
AB------E

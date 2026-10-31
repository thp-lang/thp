--TEST--
URI memory streams expose the capabilities requested by OpenMode
--FILE--
<?thp
$read = Streams::open("php://memory", OpenMode::Read);
var_dump($read->isReadable());
var_dump($read->isWritable());
var_dump($read instanceof WritableStream);
$write = Streams::open("php://temp", OpenMode::Write);
var_dump($write->isReadable());
var_dump($write->isWritable());
var_dump($write instanceof ReadableStream);
if ($write instanceof WritableStream) {
    var_dump($write->write("abc"));
}
$append = Streams::open("php://memory", OpenMode::Append);
var_dump($append->isReadable());
var_dump($append->isWritable());
$both = Streams::open("php://memory", OpenMode::ReadWriteAppend);
if ($both instanceof WritableStream) {
    $both->writeAll("a");
}
if ($both instanceof SeekableStream) {
    $both->seek(0);
}
if ($both instanceof WritableStream) {
    $both->writeAll("b");
}
if ($both instanceof SeekableStream) {
    $both->seek(0);
}
if ($both instanceof ReadableStream) {
    echo $both->readAll() . "\n";
}
try {
    Streams::open("php://memory", 99);
} catch (ValueError $error) {
    echo "invalid mode\n";
}
--EXPECT--
bool(true)
bool(false)
bool(false)
bool(false)
bool(true)
bool(false)
int(3)
bool(false)
bool(true)
ab
invalid mode

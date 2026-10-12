--TEST--
File streams read and seek without loading the file into a memory stream
--FILE--
<?thp
$file = Files::openRead("001-memory-binary-roundtrip.phpt");
var_dump($file->isReadable());
var_dump($file->isWritable());
var_dump($file->isSeekable());
var_dump($file->read(8) === "--TEST--");
var_dump($file->seek(-8, SeekFrom::Current));
var_dump($file->read(8) === "--TEST--");
--EXPECT--
bool(true)
bool(false)
bool(true)
bool(true)
int(0)
bool(true)

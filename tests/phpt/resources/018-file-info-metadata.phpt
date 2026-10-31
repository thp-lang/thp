--TEST--
SplFileInfo keeps path text and reads metadata when requested
--FILE--
<?thp
$info = new SplFileInfo("001-memory-binary-roundtrip.phpt");
echo $info->getFilename() . "\n";
echo $info->getBasename(".phpt") . "\n";
echo $info->getExtension() . "\n";
var_dump($info->getPathname() === $info->__toString());
var_dump($info->isFile());
var_dump($info->getType() === "file");
var_dump(is_int($info->getSize()));
$missing = new SplFileInfo("definitely-missing-thp-info-test");
var_dump($missing->isFile());
var_dump($missing->getSize());
--EXPECT--
001-memory-binary-roundtrip.phpt
001-memory-binary-roundtrip
phpt
bool(true)
bool(true)
bool(true)
bool(true)
bool(false)
bool(false)

--TEST--
DirectoryIterator includes dot entries and FilesystemIterator yields keyed file information
--FILE--
<?thp
$directory = new DirectoryIterator(".");
$directory->rewind();
var_dump($directory->isDot());
$directory->seek(1);
var_dump($directory->isDot());
$files = new FilesystemIterator(".");
var_dump($files->skipsDots());
var_dump($files->followsSymlinks());
$found = false;
foreach ($files as $path => $info) {
    if ($info->getFilename() === "018-file-info-metadata.phpt") {
        var_dump($path === $info->getPathname());
        $found = true;
    }
}
var_dump($found);
--EXPECT--
bool(true)
bool(true)
bool(true)
bool(false)
bool(true)
bool(true)

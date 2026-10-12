--TEST--
GlobIterator matches final path component and reports stable count
--FILE--
<?thp
$files = new GlobIterator("./0??-*.phpt");
var_dump($files->count() >= 22);
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

--TEST--
RecursiveDirectoryIterator feeds nested file information to the common recursive adapter
--FILE--
<?thp
$tree = new RecursiveDirectoryIterator("..");
$walk = new RecursiveIteratorIterator<string, SplFileInfo>($tree);
$found = false;
foreach ($walk as $info) {
    if ($info->getFilename() === "018-file-info-metadata.phpt") {
        $found = true;
        break;
    }
}
var_dump($found);
--EXPECT--
bool(true)

--TEST--
SplFileObject iterates file lines and seeks by line number
--FILE--
<?thp
$file = new SplFileObject("018-file-info-metadata.phpt");
foreach ($file as $number => $line) {
    echo $number . ":" . $line;
    if ($number === 2) {
        break;
    }
}
$file->seek(2);
var_dump($file->value() === "--FILE--\n");
var_dump($file->fgets() === "<?thp\n");
--EXPECT--
0:--TEST--
1:SplFileInfo keeps path text and reads metadata when requested
2:--FILE--
bool(true)
bool(true)

--TEST--
file metadata queries yield and missing metadata returns false
--FILE--
<?thp
scheduler_register_default();
$info = new SplFileInfo("018-file-info-metadata.phpt");
$query = async(function () use ($info): void {
    if ($info->isFile()) { echo "C"; }
});
$other = async(function (): void { echo "B"; });
echo "A";
await($query);
await($other);
$missing = new SplFileInfo("definitely-missing-async-info.phpt");
if ($missing->getSize() === false) { echo "D"; }
--EXPECT--
ABCD

--TEST--
Missing files throw OpenStreamException with their target
--FILE--
<?thp

$path = "./definitely-missing-thp-resource-test";

try {
    Files::openRead($path);
} catch (OpenStreamException $error) {
    echo $error->getTarget() . "\n";
    var_dump($error->getSystemCode() !== 0);
}
--EXPECT--
./definitely-missing-thp-resource-test
bool(true)

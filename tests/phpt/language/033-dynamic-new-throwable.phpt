--TEST--
dynamic construction preserves throwable state
--FILE--
<?thp

class CustomFailure extends Exception {}

$class: string = "CustomFailure";
try {
    $value = new $class("dynamic");
    if ($value instanceof CustomFailure) {
        throw $value;
    }
} catch (CustomFailure $error) {
    echo $error->getMessage() . "\n";
}
--EXPECT--
dynamic

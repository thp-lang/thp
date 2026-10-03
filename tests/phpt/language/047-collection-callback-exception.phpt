--TEST--
callback exception stops collection traversal and propagates unchanged
--FILE--
<?thp

try {
    vector_map([1, 2, 3], function (int $value): int {
        if ($value == 2) { throw new Exception("stop"); }
        echo $value . "\n";
        return $value;
    });
} catch (Exception $error) {
    echo $error->getMessage() . "\n";
}
--EXPECT--
1
stop

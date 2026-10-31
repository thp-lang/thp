--TEST--
iterator consumers keep generator and callback suspensions on coroutine frames
--FILE--
<?thp
function numbers(): Generator<int, int> {
    echo "A";
    yield 1;
    suspend();
    echo "C";
    yield 2;
}
function keyed(): Generator<int, int> {
    echo "D";
    yield 5 => 3;
    suspend();
    echo "E";
    yield 7 => 4;
}
scheduler_register_default();
$reader = async(function (): void {
    $first = numbers();
    $first->rewind();
    echo iterator_count($first);
    $second = numbers();
    $second->rewind();
    $values: vector<int> = iterator_to_vector($second);
    echo $values[0] . $values[1];
    $third: Iterator<int, int> = [3, 4];
    echo iterator_apply($third, function (int $value, int $key): bool {
        suspend();
        return true;
    });
    $fourth = keyed();
    $fourth->rewind();
    $entries: map<int, int> = iterator_to_map($fourth);
    echo $entries[5] . $entries[7];
    $stop: Iterator<int, int> = [8, 9];
    echo iterator_apply($stop, fn(int $value, int $key): bool => false);
    echo $stop->value();
});
$other = async(function (): void { echo "B"; });
await($reader);
await($other);
--EXPECT--
ABC2AC122DE3419

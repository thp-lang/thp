--TEST--
loop assignments invalidate refinements before repeated reads and binding writes
--FILE--
<?thp

$value: int|string = "text";
if (is_string($value)) {
    for ($value = 1; false;) {}
    var_dump(is_int($value));
}

$value = "text";
if (is_string($value)) {
    $index = 0;
    while ($index < 2) {
        var_dump(is_int($value));
        $value = 2;
        $index = $index + 1;
    }
}

$value = "text";
if (is_string($value)) {
    for ($index = 0; $index < 2; $index = $index + 1) {
        var_dump(is_int($value));
        $value = 3;
    }
}

$key: int|string = "key";
$value = "text";
if (is_string($key)) {
    if (is_string($value)) {
        foreach ([4] as $key => $value) {
            var_dump(is_int($key));
            var_dump(is_int($value));
        }
        var_dump(is_int($value));
    }
}

$value = "text";
if (is_string($value)) {
    for ($index = 0; $index < 1; $value = 5, $index = $index + 1) {}
    var_dump(is_int($value));
}

$value = "text";
if (is_string($value)) {
    for (; $value = 6, false;) {}
    var_dump(is_int($value));
}

function text(string $value): string { return $value; }
$unchanged: mixed = "unchanged";
if (is_string($unchanged)) {
    for ($index = 0; $index < 1; $index = $index + 1) {
        echo text($unchanged) . "\n";
    }
    echo text($unchanged) . "\n";
}

$value = "text";
for ($index = 0; $index < 2; $index = $index + 1) {
    if (is_string($value)) {
        echo text($value) . "\n";
    }
    $value = 7;
}
--EXPECT--
bool(true)
bool(false)
bool(true)
bool(false)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
bool(true)
unchanged
unchanged
text

--TEST--
PCRE2 matching, replacement, splitting, named captures, and byte offsets
--FILE--
<?thp
$match = preg_match('/(?<word>ab)(c)?/', 'xxab', 256);
if (is_map($match)) {
    $word = $match['word'];
    $text = $word->text();
    if (is_string($text)) { echo $text; }
    echo $word->offset();
    $missing = $match['2'];
    echo $missing->offset();
}
echo preg_replace('/(?<letter>a)/', '${letter}!', 'banana');
$parts = preg_split('/,/', 'a,,b', -1, 5);
foreach ($parts as $part) {
    $text = $part->text();
    if (is_string($text)) { echo $text; }
    echo $part->offset();
}
--EXPECT--
ab2-1ba!na!na!a0b3

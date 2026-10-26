--TEST--
PCRE2 handles absent matches, zero-width matches, limits, and split capture offsets
--FILE--
<?thp
if (is_null(preg_match('/z/', 'abc'))) { echo 'none|'; }
$matched = preg_match('/(a)?b/', 'b', 512);
if (is_map($matched)) {
    if (is_null($matched['1'])) { echo 'null|'; }
}
echo preg_replace('/a/', '!', 'banana', 0);
echo '|';
if (preg_replace('/a/u', '!', "\xff", 0) === "\xff") { echo 'binary|'; }
echo preg_replace('/(?=b)/', '!', 'ab');
echo '|';
$parts = preg_split('/(,)/', ',a,,', -1, 7);
foreach ($parts as $part) {
    $text = $part->text();
    if (is_string($text)) { echo $text; }
    echo '@';
    echo $part->offset();
    echo ';';
}
--EXPECT--
none|null|banana|binary|a!b|,@0;a@1;,@2;,@3;

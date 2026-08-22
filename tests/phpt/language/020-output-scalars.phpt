--TEST--
echo and concatenation use canonical output-scalar formatting
--FILE--
<?thp

echo "text\n";
echo 42;
echo "\n";
echo 1.0;
echo "\n";
echo -0.0;
echo "\n";
echo 0.1 + 0.2;
echo "\n";
echo 1.0 / 0.0;
echo "\n";
echo -1.0 / 0.0;
echo "\n";
echo 0.0 / 0.0;
echo "\n";
echo true;
echo "\n";
echo false;
echo "\n";
echo "[" . false . "]";
--EXPECT--
text
42
1.0
-0.0
0.30000000000000004
INF
-INF
NAN
true
false
[false]

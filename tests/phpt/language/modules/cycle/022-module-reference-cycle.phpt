--TEST--
Cross-file recursive declaration cycle is legal
--CONFIG--
[autoload]
"App\\" = "src/"
--FILE_EXTERNAL--
main.thp
--EXPECT--
0

--TEST--
Static project modules resolve imported functions
--CONFIG--
[autoload]
"App\\" = "src/"
--FILE_EXTERNAL--
main.thp
--EXPECT--
25

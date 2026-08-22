--TEST--
Unknown static module import is a compile error
--CONFIG--
[autoload]
"App\\" = "src/"
--FILE_EXTERNAL--
main.thp
--EXPECTF--
%smain.thp:2:14: error[M0003]: unknown imported symbol `App\missing`
%A

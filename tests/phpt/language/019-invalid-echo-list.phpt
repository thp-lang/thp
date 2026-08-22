--TEST--
echo accepts one string expression rather than a comma-separated list
--FILE--
<?thp

$choices: vector<int|string> = ["changed"];
echo $choices[0], "\n";
--EXPECTF--
%s019-invalid-echo-list.phpt:4:17: error[P0302]: `echo` accepts one expression; concatenate values with `.`
    4 | echo $choices[0], "\n";
      |                 ^

--TEST--
unknown named arguments report the callable and known parameter names
--FILE--
<?thp

function greet(string $name, string $suffix = "!"): string {
    return $name . $suffix;
}

echo greet(person: "Ada");
--EXPECTF--
%s008-unknown-named-argument.phpt:7:12: error[T0314]: unknown named argument `person` for `greet`
    7 | echo greet(person: "Ada");
      |            ^^^^^^
 note: known parameters: name, suffix
%s008-unknown-named-argument.phpt:7:6: error[T0316]: function `greet` is missing required argument `name`
    7 | echo greet(person: "Ada");
      |      ^^^^^^^^^^^^^^^^^^^^
 %s008-unknown-named-argument.phpt:3:16: related location
    3 | function greet(string $name, string $suffix = "!"): string {
      |                ^^^^^^^^^^^^ required parameter is declared here


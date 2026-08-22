--TEST--
trait method conflicts require an explicit insteadof selection
--FILE--
<?thp

trait First {
    public function render(): string { return "first"; }
}

trait Second {
    public function render(): string { return "second"; }
}

class Invalid {
    use First, Second;
}
--EXPECTF--
%s017-trait-conflict.phpt:12:5: error[T0027]: trait method `render` has multiple contributors
   12 |     use First, Second;
      |     ^^^^^^^^^^^^^^^^^^
 %s017-trait-conflict.phpt:4:5: related location
    4 |     public function render(): string { return "first"; }
      |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ contribution is here
 %s017-trait-conflict.phpt:8:5: related location
    8 |     public function render(): string { return "second"; }
      |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ contribution is here

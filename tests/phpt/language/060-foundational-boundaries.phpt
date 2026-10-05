--TEST--
foundational contracts reject absence, invalid counts, trace fields, and invalid serialization
--FILE--
<?thp
class BadCount implements Countable { public function count(): int { return -1; } }
class BaseCount { public function count(): int { return -1; } }
class InheritedCount extends BaseCount implements Countable {}
try { Option<int>::none()->get(); } catch (OutOfBoundsException $error) { echo "none\n"; }
try { count(new BadCount()); } catch (UnexpectedValueException $error) { echo "count\n"; }
try { (new BadCount())->count(); } catch (UnexpectedValueException $error) { echo "direct count\n"; }
try { (new InheritedCount())->count(); } catch (UnexpectedValueException $error) { echo "inherited count\n"; }
try { new TraceLine("f", -1, "p", "", null, "", null); } catch (InvalidArgumentException $error) { echo "trace\n"; }
try { new TraceLine("f", 1, "p", "", null, "", 1); } catch (InvalidArgumentException $error) { echo "trace args\n"; }
$line = new TraceLine("f", 1, "p", "", null, "", [1, 2]);
try { $line->__construct("f", 2, "p", "", null, "", null); } catch (TypeError $error) { echo "immutable\n"; }
try { serialize(new Exception("x")); } catch (InvalidArgumentException $error) { echo "object\n"; }
try { unserialize("bad"); } catch (UnexpectedValueException $error) { echo "format\n"; }
--EXPECT--
none
count
direct count
inherited count
trace
trace args
immutable
object
format

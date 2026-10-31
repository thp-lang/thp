--TEST--
reflected function and method calls suspend through coroutine frames
--FILE--
<?thp
function reflected(): int { echo "A"; suspend(); echo "C"; return 7; }
class ReflectedWorker {
    public function run(): int { echo "D"; suspend(); echo "F"; return 8; }
}
class ReflectedBuild {
    public int $value = 0;
    public function __construct() { echo "G"; suspend(); echo "I"; $this->value = 9; }
}
scheduler_register_default();
$function = new ReflectionFunction("reflected");
$method = new ReflectionMethod("ReflectedWorker", "run");
$worker = new ReflectedWorker();
$first = async(function () use ($function): void {
    $result = $function->invokeArgs([]);
    if (is_int($result)) { echo $result; }
});
$second = async(function (): void { echo "B"; });
await($first);
await($second);
$third = async(function () use ($method, $worker): void {
    $result = $method->invokeArgs($worker, []);
    if (is_int($result)) { echo $result; }
});
$fourth = async(function (): void { echo "E"; });
await($third);
await($fourth);
$class = new ReflectionClass("ReflectedBuild");
$fifth = async(function () use ($class): void {
    $result = $class->newInstanceArgs([]);
    if ($result instanceof ReflectedBuild) { echo $result->value; }
});
$sixth = async(function (): void { echo "H"; });
await($fifth);
await($sixth);
--EXPECT--
ABC7DEF8GHI9

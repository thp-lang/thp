--TEST--
cancellation through iterator_apply runs callback cleanup and ends traversal
--FILE--
<?thp
scheduler_register_default();
$job = async(function (): void {
    $cursor: Iterator<int, int> = [1, 2];
    try {
        iterator_apply($cursor, function (int $value, int $key): bool {
            try {
                echo "A";
                suspend();
            } finally {
                echo "F";
            }
            return true;
        });
    } catch (CancellationException $error) {
        echo "C";
    }
});
$canceller = async(function () use ($job): void { echo "B"; $job->cancel(); });
await($job);
await($canceller);
--EXPECT--
ABFC

--TEST--
filter callback cancellation runs cleanup and discards native scan frames
--FILE--
<?thp
scheduler_register_default();
$job = async(function (): void {
    $filter = new CallbackFilterIterator([1, 2], function (int $value, int $key): bool {
        try {
            echo "A";
            suspend();
        } finally {
            echo "F";
        }
        return true;
    });
    try {
        $filter->valid();
    } catch (CancellationException $error) {
        echo "C";
    }
});
$canceller = async(function () use ($job): void { echo "B"; $job->cancel(); });
await($job);
await($canceller);
--EXPECT--
ABFC

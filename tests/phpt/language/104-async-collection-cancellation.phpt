--TEST--
cancellation through a collection callback clears the pending traversal
--FILE--
<?thp
scheduler_register_default();
$job = async(function (): void {
    try {
        vector_map([1, 2], function (int $value): int {
            echo "A";
            suspend();
            return $value;
        });
    } catch (CancellationException $error) {
        echo "C";
    }
});
$canceller = async(function () use ($job): void { echo "B"; $job->cancel(); });
await($job);
await($canceller);
--EXPECT--
ABC

--TEST--
custom scheduler callbacks cannot suspend the request
--FILE--
<?thp
class BadScheduler implements SchedulerInterface {
    public function chooseNext(vector<int> $ready): int {
        suspend();
        return $ready[0];
    }
}
scheduler_register(new BadScheduler());
$job = async(fn(): int => 1);
await($job);
--EXPECTF--
%s121-async-scheduler-cannot-suspend.phpt:%d:%d: runtime error: uncaught Error: Async callback cannot reenter or suspend the scheduler
  at <main> (source bytes %d..%d)

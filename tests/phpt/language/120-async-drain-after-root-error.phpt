--TEST--
root failure still drains spawned work before failing the request
--FILE--
<?thp
scheduler_register_default();
async(function (): void { echo "drained\n"; });
throw new RuntimeException("root");
--EXPECTF--
drained
%s120-async-drain-after-root-error.phpt:%d:%d: runtime error: uncaught RuntimeException: root

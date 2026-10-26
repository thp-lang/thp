--TEST--
await requires Completable rather than the Awaitable marker
--FILE--
<?thp
class MarkerOnly implements Awaitable<int> {}
await(new MarkerOnly());
--EXPECTF--
%s124-async-awaitable-marker-rejected.phpt:3:%d: error[T0302]: `await` requires `Completable<T>`, found `MarkerOnly`
%A

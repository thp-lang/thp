--TEST--
throw and ordered catches enforce the sealed Throwable hierarchy
--FILE--
<?thp

class Plain {}

throw new Plain();

try {
    throw new Exception();
} catch (Throwable $general) {
} catch (Exception $specific) {
}
--EXPECTF--
%s018-throwable-boundaries.phpt:5:7: error[T0008]: thrown expressions must be `Throwable`, found `Plain`
    5 | throw new Plain();
      |       ^^^^^^^^^^^
%s018-throwable-boundaries.phpt:10:10: error[T0451]: catch `Exception` is already handled by earlier `Throwable`
   10 | } catch (Exception $specific) {
      |          ^^^^^^^^^

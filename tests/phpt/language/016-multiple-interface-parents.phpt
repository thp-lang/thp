--TEST--
interfaces reject multiple direct parents with a targeted syntax diagnostic
--FILE--
<?thp

interface First {}
interface Second {}
interface Invalid extends First, Second {}
--EXPECTF--
%s016-multiple-interface-parents.phpt:5:32: error[P0162]: an interface may extend at most one interface
    5 | interface Invalid extends First, Second {}
      |                                ^

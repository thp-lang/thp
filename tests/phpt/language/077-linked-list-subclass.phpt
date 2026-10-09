--TEST--
linked list supports user subclasses
--FILE--
<?thp
class Bag extends LinkedList<int> {
    public int $marker = 7;
}
$bag = new Bag();
$bag->push(3);
echo $bag->pop() . ":" . $bag->marker;
--EXPECT--
3:7

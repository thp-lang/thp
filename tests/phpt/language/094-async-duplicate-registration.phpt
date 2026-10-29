--TEST--
each request accepts one scheduler registration
--FILE--
<?thp
echo scheduler_is_registered() . "\n";
scheduler_register_default();
try { scheduler_register_default(); }
catch (Error $error) { echo "duplicate\n"; }
echo scheduler_is_registered() . "\n";
--EXPECT--
false
duplicate
true

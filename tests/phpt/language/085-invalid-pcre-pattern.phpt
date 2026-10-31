--TEST--
invalid PCRE2 patterns report the call site
--FILE--
<?thp
echo preg_replace('/([/', 'x', 'abc');
--EXPECTF--
%s085-invalid-pcre-pattern.phpt:2:6: runtime error: invalid PCRE2 pattern: %s

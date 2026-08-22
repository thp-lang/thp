# Reimporting PHP 8.5 fixtures

THP does not track the inactive upstream PHP fixtures in its main tree. They are
migration inputs, not executable THP specifications and not evidence of PHP
compatibility.

The last audited import used the non-extension suites from
[`php-src` 8.5.6](https://github.com/php/php-src/tree/php-8.5.6):

- it contained 6,172 PHPT files plus adjacent support files;
- upstream `Zend/tests/` became `tests/phpt/php/zend/`;
- upstream `tests/` became `tests/phpt/php/core/`;
- extensions and SAPI suites were excluded;
- the source archive was `php-src-php-8.5.6.tar.gz`;
- its SHA-256 was
  `cbb8833bb72ab11af5406c597cc6075c599e611fe965dd9749867e95182cebf9`;
- upstream PHP test material remains under the
  [PHP License 3.01](https://github.com/php/php-src/blob/php-8.5.6/LICENSE).

To recreate the import from a verified extracted source release, ensure the
destination does not exist and run:

```sh
python3 tests/phpt/port_php_85.py \
  /path/to/php-src-php-8.5.6 \
  tests/phpt/php
```

The importer performs only a mechanical discovery pass. Any fixture promoted
to a native THP specification must be reviewed, minimized, renamed to the
numbered kebab-case convention, and placed outside `tests/phpt/php/` with an
explicitly compatible license.

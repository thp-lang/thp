---
kind: module
id: std.extensions
title: Bundled extensions
summary: Bundled PCRE2 regex functions and proposed optional extensions.
module: extensions
order: 80
status: experimental
availability: partial
notice: The PCRE2 regex functions are implemented. The other optional extension
  entries below remain design placeholders.
---

The bundled PCRE2 regex extension is always available without configuration:
[`preg_match()`](thp:std.extensions.preg_match),
[`preg_replace()`](thp:std.extensions.preg_replace), and
[`preg_split()`](thp:std.extensions.preg_split). Offset results use
[`RegexCapture`](thp:std.extensions.RegexCapture).

The remaining entries catalog proposed optional extensions. They are design
placeholders and do not promise PHP compatibility.

| Extension               | Dependency model        | Status             |
| ----------------------- | ----------------------- | ------------------ |
| Apache                  | Bundled dependency only | Design placeholder |
| BCMath                  | Bundled dependency only | Design placeholder |
| Bzip2                   | External dependencies   | Design placeholder |
| Calendar                | Bundled dependency only | Design placeholder |
| COM                     | Bundled dependency only | Design placeholder |
| Ctype                   | Bundled dependency only | Design placeholder |
| cURL                    | External dependencies   | Design placeholder |
| DBA                     | Bundled dependency only | Design placeholder |
| DOM                     | External dependencies   | Design placeholder |
| Enchant                 | External dependencies   | Design placeholder |
| Exif                    | Bundled dependency only | Design placeholder |
| FFI                     | Bundled dependency only | Design placeholder |
| Fileinfo                | Bundled dependency only | Design placeholder |
| Filter                  | Bundled dependency only | Design placeholder |
| FTP                     | Bundled dependency only | Design placeholder |
| GD                      | Bundled dependency only | Design placeholder |
| Gettext                 | External dependencies   | Design placeholder |
| GMP                     | External dependencies   | Design placeholder |
| iconv                   | Bundled dependency only | Design placeholder |
| intl                    | Bundled dependency only | Design placeholder |
| LDAP                    | External dependencies   | Design placeholder |
| libxml                  | External dependencies   | Design placeholder |
| LiteSpeed               | Bundled dependency only | Design placeholder |
| Mbstring                | Bundled dependency only | Design placeholder |
| Mhash                   | Bundled dependency only | Design placeholder |
| MySQLi                  | External dependencies   | Design placeholder |
| mysqlnd                 | External dependencies   | Design placeholder |
| ODBC                    | External dependencies   | Design placeholder |
| OpenSSL                 | External dependencies   | Design placeholder |
| PCNTL                   | Bundled dependency only | Design placeholder |
| PDO                     | Bundled dependency only | Design placeholder |
| PDO DBLIB driver        | External dependencies   | Design placeholder |
| PDO Firebird driver     | External dependencies   | Design placeholder |
| PDO MySQL driver        | External dependencies   | Design placeholder |
| PDO ODBC and DB2 driver | External dependencies   | Design placeholder |
| PDO PostgreSQL driver   | External dependencies   | Design placeholder |
| PDO SQLite driver       | External dependencies   | Design placeholder |
| Phar                    | Bundled dependency only | Design placeholder |
| phpdbg                  | Bundled dependency only | Design placeholder |
| POSIX                   | Bundled dependency only | Design placeholder |
| PostgreSQL              | External dependencies   | Design placeholder |
| Readline                | External dependencies   | Design placeholder |
| Semaphore               | Bundled dependency only | Design placeholder |
| Sessions                | Bundled dependency only | Design placeholder |
| Shared Memory           | Bundled dependency only | Design placeholder |
| SimpleXML               | External dependencies   | Design placeholder |
| SNMP                    | External dependencies   | Design placeholder |
| SOAP                    | External dependencies   | Design placeholder |
| Sockets                 | Bundled dependency only | Design placeholder |
| Sodium                  | External dependencies   | Design placeholder |
| SQLite3                 | Bundled dependency only | Design placeholder |
| Tidy                    | External dependencies   | Design placeholder |
| Tokenizer               | Bundled dependency only | Design placeholder |
| XML Parser              | External dependencies   | Design placeholder |
| XMLReader               | External dependencies   | Design placeholder |
| XMLWriter               | External dependencies   | Design placeholder |
| XSL                     | External dependencies   | Design placeholder |
| Zip                     | External dependencies   | Design placeholder |
| Zlib                    | Bundled dependency only | Design placeholder |

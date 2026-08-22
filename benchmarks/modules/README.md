# Static-module comparison benchmark

This opt-in suite compares equivalent statically named PHP and THP programs.
It is not a CI performance gate. Record the CPU, memory, kernel, filesystem,
Rust toolchain, THP commit, PHP build/configuration, Composer version, and exact
commands beside every published result.

An early local run observed THP at over 17× the speed and about one tenth the
peak memory of the compared PHP invocation. That is preliminary information
from one synthetic comparison, not a representative benchmark or a general
performance promise. Formal results will be added only with the complete
environment record, exact commands, and exported raw samples described below.

Prerequisites are a release THP build, PHP 8.5, Composer, Hyperfine, and
`/usr/bin/time`.

```sh
cargo build --release -p thp-cli
composer --working-dir=benchmarks/modules/php install --no-dev --classmap-authoritative
mkdir -p /tmp/thp-module-opcache /tmp/php-module-opcache
target/release/thp cache-warm \
  --project=benchmarks/modules/thp \
  --opcache=/tmp/thp-module-opcache \
  main.thp
php \
  -d opcache.enable_cli=1 \
  -d opcache.file_cache=/tmp/php-module-opcache \
  -d opcache.validate_timestamps=0 \
  benchmarks/modules/php/main.php
```

Measure process startup and steady execution separately. Hyperfine JSON contains
the raw samples needed to report median and p95:

```sh
hyperfine --warmup 10 --runs 50 --export-json=/tmp/modules.json \
  'target/release/thp run --project=benchmarks/modules/thp --opcache=/tmp/thp-module-opcache main.thp' \
  'target/release/thp run --frozen --engine=vm --project=benchmarks/modules/thp --opcache=/tmp/thp-module-opcache main.thp' \
  'target/release/thp run --frozen --engine=auto --project=benchmarks/modules/thp --opcache=/tmp/thp-module-opcache main.thp' \
  'php -d opcache.enable_cli=1 -d opcache.file_cache=/tmp/php-module-opcache -d opcache.validate_timestamps=0 -d opcache.jit=0 benchmarks/modules/php/main.php' \
  'php -d opcache.enable_cli=1 -d opcache.file_cache=/tmp/php-module-opcache -d opcache.validate_timestamps=0 -d opcache.jit=tracing benchmarks/modules/php/main.php'
```

Collect THP phase timings with `--metrics=json`, peak RSS with
`/usr/bin/time -v`, and artifact sizes with:

```sh
du -ab /tmp/thp-module-opcache /tmp/php-module-opcache
```

For incremental runs, change only the body and then only the signature in
`Calculator`; record the `ProjectCompilation.units[].cache` rebuilt/reused
counts. Restore the fixture before publishing results.

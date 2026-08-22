---
name: run-project-benchmarks
description: Run and report reproducible THP project performance benchmarks. Use when asked to run benchmarks, compare THP with PHP or another runtime, measure startup or frozen execution, collect Hyperfine samples, phase metrics, peak RSS, cache artifact sizes, or incremental compilation/cache behavior.
---

# Run Project Benchmarks

Run the repository benchmark protocol without turning noisy local measurements
into unsupported performance claims. Preserve raw samples, the execution
environment, exact commands, and workload equivalence.

## Establish authority and scope

1. Read the repository `AGENTS.md`.
2. Discover benchmark files with `rg --files benchmarks`.
3. Read the selected benchmark's README completely. Treat it as authoritative
   for prerequisites, warm-up, commands, workloads, and required reporting.
4. Do not add hardware-sensitive thresholds or infer a CI performance gate.
5. Inspect `git status --short`. Preserve all existing changes and record a
   dirty worktree rather than cleaning it.

If several suites match an ambiguous request, run the smallest authoritative
suite first and identify what remains. Do not mutate benchmark fixtures for an
optional incremental scenario unless the request includes it or the benchmark
README requires it.

## Preflight the environment

Verify every declared prerequisite before measuring. For the static-module
suite this normally includes:

```sh
command -v php
command -v composer
command -v hyperfine
command -v /usr/bin/time
php -v
composer --version
hyperfine --version
rustc --version
cargo --version
```

If a tool is missing, report it and request approval before installing it.
Never silently replace a required measurement tool. Check current load, memory,
swap, CPU scaling, and thermal or power constraints when available. Warn about
material interference, but continue when the user asked for an immediate run.

Capture provenance before and after the run:

```sh
bash .agents/skills/run-project-benchmarks/scripts/capture-environment.sh
```

Record at least the date/time zone, CPU, memory, kernel, filesystem, Rust/PHP/
Composer/Hyperfine versions, loaded PHP configuration, commit, dirty status,
and release-binary checksum.

## Prepare and validate

1. Build the exact release target required by the README.
2. Install benchmark-local dependencies using the prescribed command.
3. Inspect cache destinations before using them. Prefer fresh run-specific
   directories under `/tmp`; never delete a pre-existing cache of unknown
   ownership.
4. Warm every compared artifact exactly as required.
5. Execute every command once outside the timing harness.
6. Require successful exits and byte-equivalent observable output before
   comparing performance.

List dependency artifacts created in the worktree, such as `vendor/` or a lock
file. Do not remove pre-existing files or user changes.

## Measure timing

Run the README's prescribed command first and export Hyperfine JSON. Keep all
raw samples.

- Use the declared warm-up and run counts; do not shorten them silently.
- Run competitors sequentially, never concurrently.
- Separate source/project startup from frozen or steady execution.
- Do not discard outliers. Report them and rerun only as an additional sample.
- If Hyperfine warns that commands are below its shell-resolution range, keep
  the prescribed result and add a second `--shell=none` run. Do not replace or
  overwrite the original JSON.
- Confirm which engine `auto` actually selected from phase metrics. Do not label
  VM fallback as JIT.

Summarize raw JSON with:

```sh
node .agents/skills/run-project-benchmarks/scripts/summarize-hyperfine.mjs \
  /tmp/modules.json
```

Use the median as the primary statistic, nearest-rank p95 for tail latency, and
mean ± standard deviation as secondary context. Compute speed ratios from the
same statistic on equivalent workloads.

## Collect supporting measurements

Collect each category separately from Hyperfine:

1. **THP phases:** run with `--metrics=json`, redirect program output and metrics
   to different files, validate output again, and aggregate repeated stage
   names.
2. **Peak RSS:** run every command sequentially through `/usr/bin/time -v`.
   Treat each result as a single sample unless repeated explicitly.
3. **Artifacts:** use the README's `du` command and report directory totals as
   well as important components.
4. **Correctness:** retain the expected and actual output or its checksum.

Do not equate internal VM time with whole-process latency. Do not describe a
tiny startup fixture as general language throughput.

## Test incremental behavior

When incremental testing is in scope:

1. Preserve the exact original fixture contents and hashes without using
   destructive Git restoration.
2. Change only a function body and record each unit's rebuilt/reused cache
   fields.
3. Restore it, then change only its signature and record the same fields.
4. Restore the fixture even after a failed command and verify `git status`.

First confirm that the current CLI or harness exposes
`ProjectCompilation.units[].cache`. If it does not, report the instrumentation
gap; do not change production code merely to manufacture benchmark output
unless the user authorizes that expansion.

## Report results

Provide:

- a table with command/variant, median, p95, mean ± σ, and peak RSS;
- speed and memory ratios using clearly named baselines;
- phase timing and artifact-size summaries;
- exact commands or a direct link to the authoritative command block;
- environment and repository provenance;
- raw JSON paths and checksums;
- correctness confirmation;
- caveats including load, swap, frequency scaling, outliers, dirty state, tiny
  workloads, VM fallback, or single-sample RSS;
- generated worktree artifacts and any optional scenarios not run.

Lead with what was measured, not a broad claim that THP is faster than PHP.

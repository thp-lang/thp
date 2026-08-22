#!/usr/bin/env node

import { readFileSync } from "node:fs";

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(2);
}

function nearestRank(sorted, percentile) {
  const index = Math.max(0, Math.ceil(sorted.length * percentile) - 1);
  return sorted[index];
}

const paths = process.argv.slice(2);
if (paths.length === 0) {
  fail("usage: summarize-hyperfine.mjs HYPERFINE.json [...]");
}

process.stdout.write(
  [
    "file",
    "command",
    "runs",
    "median_ms",
    "p95_ms",
    "mean_ms",
    "stddev_ms",
    "min_ms",
    "max_ms",
  ].join("\t") + "\n",
);

for (const path of paths) {
  let document;
  try {
    document = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`cannot read Hyperfine JSON ${path}: ${error.message}`);
  }

  if (!Array.isArray(document.results)) {
    fail(`invalid Hyperfine JSON ${path}: missing results array`);
  }

  for (const result of document.results) {
    if (!Array.isArray(result.times) || result.times.length === 0) {
      fail(`invalid Hyperfine result in ${path}: missing timing samples`);
    }
    const times = [...result.times].sort((left, right) => left - right);
    const milliseconds = (seconds) => (seconds * 1000).toFixed(3);
    process.stdout.write(
      [
        path,
        result.command,
        times.length,
        milliseconds(result.median),
        milliseconds(nearestRank(times, 0.95)),
        milliseconds(result.mean),
        milliseconds(result.stddev),
        milliseconds(result.min),
        milliseconds(result.max),
      ].join("\t") + "\n",
    );
  }
}

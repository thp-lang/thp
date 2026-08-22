#!/usr/bin/env bash

set -eu

run_if_available() {
    if command -v "$1" >/dev/null 2>&1; then
        "$@"
    else
        echo "missing command: $1"
    fi
}

echo "captured_at=$(date --iso-8601=seconds)"
echo "timezone=$(date +%Z)"
echo "commit=$(git rev-parse HEAD)"
echo "worktree_status:"
git status --short
echo "kernel:"
uname -a
echo "cpu:"
run_if_available lscpu
echo "memory:"
run_if_available free -h
echo "load:"
cat /proc/loadavg
echo "filesystem:"
run_if_available findmnt -T . -o TARGET,SOURCE,FSTYPE,OPTIONS
echo "rust:"
run_if_available rustc --version
run_if_available cargo --version
echo "php:"
run_if_available php -v
run_if_available php --ini
echo "composer:"
run_if_available composer --version
echo "hyperfine:"
run_if_available hyperfine --version
echo "gnu_time:"
if [ -x /usr/bin/time ]; then
    /usr/bin/time --version
else
    echo "missing command: /usr/bin/time"
fi

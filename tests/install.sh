#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/release/thp-9.9.9-linux-x86_64/bin" "$fixture/mockbin"
cat > "$fixture/release/thp-9.9.9-linux-x86_64/bin/thp" <<'THP'
#!/bin/sh
echo 'thp 9.9.9'
THP
chmod +x "$fixture/release/thp-9.9.9-linux-x86_64/bin/thp"
for notice in LICENSE LICENSE-BINARY LICENSING.md NOTICE THIRD-PARTY-NOTICES INSTALL.md; do
  echo "$notice" > "$fixture/release/thp-9.9.9-linux-x86_64/$notice"
done
tar -czf "$fixture/release/thp-9.9.9-linux-x86_64.tar.gz" -C "$fixture/release" thp-9.9.9-linux-x86_64
(cd "$fixture/release" && sha256sum thp-9.9.9-linux-x86_64.tar.gz > SHA256SUMS)
cat > "$fixture/release/releases.json" <<'JSON'
[
  {
    "tag_name": "thp-lsp-v10.0.0"
  },
  {
    "tag_name": "v10.0.0",
    "prerelease": true
  },
  {
    "prerelease": false,
    "tag_name": "v9.9.9"
  }
]
JSON
cat > "$fixture/mockbin/curl" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
while (($#)); do
  case "$1" in
    -o) output=$2; shift 2 ;;
    https:*) url=$1; shift ;;
    *) shift ;;
  esac
done
case "$url" in
  *releases\?*) source=releases.json ;;
  *) source=${url##*/} ;;
esac
cp "$MOCK_RELEASE_DIR/$source" "$output"
MOCK
chmod +x "$fixture/mockbin/curl"

unset THP_VERSION
export MOCK_RELEASE_DIR="$fixture/release" THP_INSTALL_DIR="$fixture/install"
PATH="$fixture/mockbin:$PATH" bash "$root/install.sh" > "$fixture/output"
[[ $("$fixture/install/thp" --version) == 'thp 9.9.9' ]]
[[ -f "$fixture/install/thp-notices/LICENSE-BINARY" ]]
sed 's/^  /    /' "$fixture/release/releases.json" > "$fixture/release/releases.indented"
mv "$fixture/release/releases.indented" "$fixture/release/releases.json"
PATH="$fixture/mockbin:$PATH" bash "$root/install.sh" > "$fixture/output"

mkdir -p "$fixture/directory-install/thp"
if PATH="$fixture/mockbin:$PATH" THP_INSTALL_DIR="$fixture/directory-install" bash "$root/install.sh" > "$fixture/output" 2>&1; then
  echo 'installer accepted a directory in place of thp' >&2
  exit 1
fi
[[ -d "$fixture/directory-install/thp" ]]

echo 'new LICENSE' > "$fixture/release/thp-9.9.9-linux-x86_64/LICENSE"
mv "$fixture/release/thp-9.9.9-linux-x86_64/NOTICE" "$fixture/release/thp-9.9.9-linux-x86_64/NOTICE.missing"
tar -czf "$fixture/release/thp-9.9.9-linux-x86_64.tar.gz" -C "$fixture/release" thp-9.9.9-linux-x86_64
(cd "$fixture/release" && sha256sum thp-9.9.9-linux-x86_64.tar.gz > SHA256SUMS)
if PATH="$fixture/mockbin:$PATH" bash "$root/install.sh" > "$fixture/output" 2>&1; then
  echo 'installer accepted an incomplete notice set' >&2
  exit 1
fi
[[ $(cat "$fixture/install/thp-notices/LICENSE") == LICENSE ]] || {
  echo 'failed update replaced an existing notice' >&2
  exit 1
}
mv "$fixture/release/thp-9.9.9-linux-x86_64/NOTICE.missing" "$fixture/release/thp-9.9.9-linux-x86_64/NOTICE"
tar -czf "$fixture/release/thp-9.9.9-linux-x86_64.tar.gz" -C "$fixture/release" thp-9.9.9-linux-x86_64
(cd "$fixture/release" && sha256sum thp-9.9.9-linux-x86_64.tar.gz > SHA256SUMS)
mkdir -p "$fixture/failmv"
cat > "$fixture/failmv/mv" <<'MOCK'
#!/usr/bin/env bash
for arg; do target=$arg; done
if [[ $target == "$THP_INSTALL_DIR/thp" ]]; then exit 1; fi
exec /usr/bin/mv "$@"
MOCK
chmod +x "$fixture/failmv/mv"
if PATH="$fixture/failmv:$fixture/mockbin:$PATH" bash "$root/install.sh" > "$fixture/output" 2>&1; then
  echo 'installer accepted a failed executable replacement' >&2
  exit 1
fi
[[ $(cat "$fixture/install/thp-notices/LICENSE") == LICENSE ]] || {
  echo 'failed executable replacement did not restore notices' >&2
  exit 1
}
[[ $("$fixture/install/thp" --version) == 'thp 9.9.9' ]]

cat > "$fixture/release/thp-9.9.9-linux-x86_64/bin/thp" <<'THP'
#!/bin/sh
exit 42
THP
tar -czf "$fixture/release/thp-9.9.9-linux-x86_64.tar.gz" -C "$fixture/release" thp-9.9.9-linux-x86_64
(cd "$fixture/release" && sha256sum thp-9.9.9-linux-x86_64.tar.gz > SHA256SUMS)
if PATH="$fixture/mockbin:$PATH" bash "$root/install.sh" > "$fixture/output" 2>&1; then
  echo 'installer accepted a broken executable' >&2
  exit 1
fi
[[ $("$fixture/install/thp" --version) == 'thp 9.9.9' ]] || {
  echo 'failed update replaced the working executable' >&2
  exit 1
}

printf '%064d  thp-9.9.9-linux-x86_64.tar.gz\n' 0 > "$fixture/release/SHA256SUMS"
if PATH="$fixture/mockbin:$PATH" bash "$root/install.sh" > "$fixture/output" 2>&1; then
  echo 'installer accepted a wrong checksum' >&2
  exit 1
fi
grep -q 'checksum mismatch' "$fixture/output"
echo 'installer smoke passed'

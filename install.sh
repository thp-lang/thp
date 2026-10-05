#!/usr/bin/env bash
set -euo pipefail

fail() { echo "THP install: $*" >&2; exit 1; }

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    getconf GNU_LIBC_VERSION >/dev/null 2>&1 || fail "the Linux archive requires glibc"
    platform=linux-x86_64 ;;
  Darwin-x86_64) platform=macos-x86_64 ;;
  Darwin-arm64) platform=macos-arm64 ;;
  *) fail "no release archive for $(uname -s)-$(uname -m)" ;;
esac

command -v curl >/dev/null || fail "curl is required"
tmp=$(mktemp -d)
staged=
staged_notices=
trap 'rm -rf "$tmp"; if [[ -n $staged ]]; then rm -f "$staged"; fi; if [[ -n $staged_notices ]]; then rm -rf "$staged_notices"; fi' EXIT

if [[ -n ${THP_VERSION:-} ]]; then
  version=$THP_VERSION
else
  curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL \
    'https://api.github.com/repos/thp-lang/thp/releases?per_page=100' \
    -o "$tmp/releases.json"
  version=$(awk '
    /^[[:space:]]*\{/ {
      position = match($0, /\{/)
      if (!release_position || position == release_position) {
        release_position = position
        tag = ""
        stable = 0
      }
    }
    /^[[:space:]]*"tag_name": / {
      tag = $0
      sub(/^.*"tag_name": "v/, "", tag)
      sub(/".*$/, "", tag)
      if (tag !~ /^[0-9]+\.[0-9]+\.[0-9]+$/) tag = ""
    }
    /^[[:space:]]*"prerelease": false[, ]*$/ { stable = 1 }
    /^[[:space:]]*\}/ && match($0, /\}/) == release_position && stable && tag != "" { print tag; exit }
  ' "$tmp/releases.json")
fi
[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "no stable THP release found"

archive="thp-$version-$platform.tar.gz"
base="https://github.com/thp-lang/thp/releases/download/v$version"
curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL "$base/$archive" -o "$tmp/$archive"
curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"
expected=$(awk -v name="$archive" '$2 == name { print $1 }' "$tmp/SHA256SUMS")
[[ $expected =~ ^[[:xdigit:]]{64}$ ]] || fail "checksum missing for $archive"
if [[ $platform == macos-* ]]; then
  actual=$(shasum -a 256 "$tmp/$archive" | awk '{ print $1 }')
else
  actual=$(sha256sum "$tmp/$archive" | awk '{ print $1 }')
fi
[[ $actual == "$expected" ]] || fail "checksum mismatch for $archive"

tar -xzf "$tmp/$archive" -C "$tmp"
bin_dir=${THP_INSTALL_DIR:-"$HOME/.local/bin"}
[[ $bin_dir == /* ]] || fail "THP_INSTALL_DIR must be an absolute path"
release_dir="$tmp/thp-$version-$platform"
[[ ! -d "$bin_dir/thp" ]] || fail "$bin_dir/thp is a directory"
mkdir -p "$bin_dir"
staged=$(mktemp "$bin_dir/.thp.XXXXXXXX")
install -m 755 "$release_dir/bin/thp" "$staged"
installed_version=$("$staged" --version)
staged_notices=$(mktemp -d "$bin_dir/.thp-notices.XXXXXXXX")
cp "$release_dir"/{LICENSE,LICENSE-BINARY,LICENSING.md,NOTICE,THIRD-PARTY-NOTICES,INSTALL.md} "$staged_notices/"
old_notices=
if [[ -e "$bin_dir/thp-notices" ]]; then
  old_notices=$(mktemp -d "$bin_dir/.thp-notices-old.XXXXXXXX")
  mv "$bin_dir/thp-notices" "$old_notices/notices" || {
    rmdir "$old_notices"
    fail "cannot back up notices"
  }
fi
restore_notices() {
  rm -rf "$bin_dir/thp-notices"
  if [[ -n $old_notices ]]; then
    mv "$old_notices/notices" "$bin_dir/thp-notices"
    rmdir "$old_notices"
  fi
  fail "$1"
}
mv "$staged_notices" "$bin_dir/thp-notices" || restore_notices "cannot install notices"
staged_notices=
mv -f "$staged" "$bin_dir/thp" || restore_notices "cannot install thp"
staged=
if [[ -n $old_notices ]]; then rm -rf "$old_notices"; fi
echo "$installed_version"
if [[ :$PATH: != *:"$bin_dir":* ]]; then
  echo "Add $bin_dir to PATH, then open a new shell: export PATH=\"$bin_dir:\$PATH\""
fi

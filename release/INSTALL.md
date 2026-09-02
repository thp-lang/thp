# Installing THP

THP is experimental and intended for command-line evaluation.

Official binary archives contain `bin/thp` (`bin/thp.exe` on Windows). These
identified official binaries are additionally offered under the included MIT
`LICENSE-BINARY`. Repository source remains under Apache-2.0; see
`LICENSING.md`.

## Choose an archive

Download the archive for your platform from the same GitHub release:

| Platform            | Archive suffix        |
| ------------------- | --------------------- |
| Linux x86-64, glibc | `linux-x86_64.tar.gz` |
| macOS x86-64        | `macos-x86_64.tar.gz` |
| macOS Apple silicon | `macos-arm64.tar.gz`  |
| Windows x86-64      | `windows-x86_64.zip`  |

Also download these two files from that release:

- `SHA256SUMS`
- `SHA256SUMS.sigstore.json`

Keep the archive and both verification files in the same directory.

If you are reading this guide from an already extracted archive, return to the
directory containing the original downloaded archive before verifying it.

## Authenticate the checksum manifest

Install Cosign, then verify that `SHA256SUMS` was signed by the THP release
workflow:

```sh
cosign verify-blob \
  --bundle SHA256SUMS.sigstore.json \
  --certificate-identity-regexp '^https://github\.com/thp-lang/thp/\.github/workflows/release\.yml@refs/tags/v[0-9][0-9A-Za-z.+-]*$' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  SHA256SUMS
```

Continue only if Cosign reports successful verification.

The signature authenticates the checksum manifest. The next step verifies that
the downloaded archive matches the authenticated checksum.

## Verify the downloaded archive

Replace the example version with the version you downloaded.

### Linux

```sh
THP_ARCHIVE=thp-0.2.0-linux-x86_64.tar.gz

awk -v archive="$THP_ARCHIVE" '$2 == archive { print }' SHA256SUMS |
  sha256sum --check -
```

### macOS x86-64

```sh
THP_ARCHIVE=thp-0.2.0-macos-x86_64.tar.gz

awk -v archive="$THP_ARCHIVE" '$2 == archive { print }' SHA256SUMS |
  shasum --algorithm 256 --check -
```

### macOS Apple silicon

```sh
THP_ARCHIVE=thp-0.2.0-macos-arm64.tar.gz

awk -v archive="$THP_ARCHIVE" '$2 == archive { print }' SHA256SUMS |
  shasum --algorithm 256 --check -
```

### Windows PowerShell

```powershell
$ThpArchive = "thp-0.2.0-windows-x86_64.zip"
$ChecksumLine = Get-Content SHA256SUMS |
    Where-Object { $_ -match "\s+$([regex]::Escape($ThpArchive))$" }

if (-not $ChecksumLine) {
    throw "No checksum was published for $ThpArchive"
}

$Expected = ($ChecksumLine -split "\s+")[0].ToLowerInvariant()
$Actual = (Get-FileHash -Algorithm SHA256 $ThpArchive).Hash.ToLowerInvariant()

if ($Actual -ne $Expected) {
    throw "Checksum verification failed for $ThpArchive"
}

Write-Host "Checksum verified for $ThpArchive"
```

Do not install or run the executable if verification fails.

## Extract and install

Extract the verified archive. It contains a versioned platform directory with
the executable under `bin/`.

Move `bin/thp` (`bin/thp.exe` on Windows) to a directory already on `PATH`, or
add the extracted `bin/` directory to `PATH`.

Confirm the installation:

```sh
thp --version
thp --help
```

## Run a program

Create `hello.thp`:

```thp
<?thp

$name: string = "world";
echo "Hello, " . $name . "!\n";
```

Type-check and run it:

```sh
thp check hello.thp
thp run hello.thp
```

Expected output:

```text
Hello, world!
```

THP v0.2.0 is not production-ready, is not a PHP-compatible replacement, and
does not execute through PHP's engine.

## Licensing

Repository source is licensed under Apache-2.0. Specifically identified
official binaries in this archive are additionally offered under MIT.

See the included files for details:

- `LICENSE`
- `LICENSE-BINARY`
- `LICENSING.md`
- `NOTICE`
- `THIRD-PARTY-NOTICES`

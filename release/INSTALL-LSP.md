# Install thp-lsp

`thp-lsp` 0.1.0 is an experimental standalone language server. It is released
separately from the THP compiler and is not published to crates.io.

Extract the archive for your platform and add its `bin` directory to `PATH`.
Then configure an LSP client to start either `thp-lsp` or `thp-lsp --stdio` for
the `thp` language ID and `.thp` files.

Verify the binary with:

```sh
thp-lsp --version
thp-lsp --help
```

Release assets include `SHA256SUMS` and a Sigstore bundle. Verify the signed
manifest, then the archive checksum:

```sh
cosign verify-blob \
  --bundle SHA256SUMS.sigstore.json \
  --certificate-identity-regexp '^https://github\.com/thp-lang/thp/\.github/workflows/release-lsp\.yml@refs/tags/thp-lsp-v[0-9][0-9A-Za-z.+-]*$' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  SHA256SUMS

sha256sum --check SHA256SUMS
```

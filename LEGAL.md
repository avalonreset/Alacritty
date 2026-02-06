# Legal / Distribution Notes

This file is intended as practical guidance for redistributing this fork.
It is **not legal advice**.

## Project License (Upstream)

Upstream Alacritty is dual-licensed under:

- Apache License 2.0 (`LICENSE-APACHE`)
- MIT License (`LICENSE-MIT`)

As a fork, this repository keeps those license files in-tree.

## Third-Party Dependencies

Alacritty depends on many third-party crates and system libraries.
When you redistribute binaries, you should also redistribute third-party license notices.

This repo includes a generated notice file:

- `THIRD_PARTY_NOTICES.html`

You can regenerate it with:

```bash
cargo install --locked cargo-about
cargo about generate --workspace --locked about.hbs -o THIRD_PARTY_NOTICES.html
```

Offline mode (no network):

```bash
cargo about generate --workspace --locked --offline about.hbs -o THIRD_PARTY_NOTICES.html
```

Convenience scripts are provided:

- `scripts/generate-third-party-notices.ps1`
- `scripts/generate-third-party-notices.sh`

## When Sharing Builds (Practical Checklist)

If you share a `.zip`, `.exe`, or `.msi` with friends/community, include:

- `LICENSE-APACHE`
- `LICENSE-MIT`
- `THIRD_PARTY_NOTICES.html`

And make the corresponding source available (for example by linking to this GitHub repository and the exact commit/tag you built from).

## Windows MSI Packaging

The WiX installer definition is in `alacritty/windows/wix/alacritty.wxs`.
This fork’s MSI packaging is configured to install the license and notice files alongside the executable so they travel with the installation.


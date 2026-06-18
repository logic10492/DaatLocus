<div align="center">

<img src="assets/logo.svg" alt="Daat Locus Logo" style="width:220px; height:auto;" />

# Daat Locus

[![简体中文][readme-cn-badge]][readme-cn-url]
[![Crates.io][crates-badge]][crates-url]
[![CI][ci-badge]][ci-url]
[![License][license-badge]][license-url]

An agent runtime that truly has experience.

</div>

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge
[readme-cn-url]: README_zh-CN.md
[crates-badge]: https://img.shields.io/crates/v/daat-locus?style=for-the-badge
[crates-url]: https://crates.io/crates/daat-locus
[ci-badge]: https://img.shields.io/github/actions/workflow/status/shadow3aaa/DaatLocus/ci.yml?style=for-the-badge&label=CI
[ci-url]: https://github.com/shadow3aaa/DaatLocus/actions/workflows/ci.yml
[license-badge]: https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=for-the-badge
[license-url]: LICENSE

## What Is This?

Daat Locus is a long-running local, tool-driven agent runtime.

It is built for work that becomes better through history: maintaining the same
project over time, repeatedly handling the same class of task, remembering
practical experience, and turning that experience into reusable runtime
structure.

Daat Locus is not a one-shot chatbot wrapper. External input enters the runtime
as structured work, the model makes semantic decisions, and real-world changes
happen only through explicit tools.

## Quick Start

The recommended install path is `cargo-binstall`, which installs the prebuilt
GitHub Release binary for your platform. Normal installs do not need Python,
`uv`, or PyInstaller.

```bash
cargo install cargo-binstall
cargo binstall daat-locus
```

You can also download the matching archive directly from
[GitHub Releases][releases-url], extract it, and place `daat-locus` on your
`PATH`.

On first launch, Daat Locus opens an interactive setup flow.

### Source Builds

`cargo install daat-locus` is available from crates.io. Source builds require
Bun because `build.rs` builds and embeds the WebUI.

```bash
git clone https://github.com/shadow3aaa/DaatLocus
cd DaatLocus
cargo run --locked
```

`cargo build` and `cargo run` build the WebUI through `build.rs` and embed the
generated assets into the daemon by default. For a release-style local binary,
run `cargo build -p daat-locus --release --locked` directly.

[releases-url]: https://github.com/shadow3aaa/DaatLocus/releases

## Common Entry Points

```bash
daat-locus run                 # open the foreground runtime flow
daat-locus code <project-dir>  # select or create a project-scoped session
daat-locus attach              # attach to an existing daemon
daat-locus send "..."          # send one message and wait for a reply
daat-locus config              # open the interactive config menu
```

## Documentation

- [简体中文 README](README_zh-CN.md)
- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Contributing](CONTRIBUTING.md)
- [Builtin SOP primitive specs](workflows/README.md)

## License

Daat Locus is licensed under the [Apache License 2.0](LICENSE).

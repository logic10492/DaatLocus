# DeepSWE Benchmark

DeepSWE evaluates whether an agent can modify code in an isolated Linux
repository at `/app` according to `instruction.md`, then create a commit. The
runner extracts the commit diff into `/logs/artifacts/model.patch`, applies it
inside a clean verifier container, and runs hidden tests.

References:

- <https://github.com/datacurve-ai/deep-swe>
- <https://deepswe.datacurve.ai/run>

## Running

Use this flow to run DeepSWE against Daat Locus.

### Prerequisites

- Install and start Docker.
- Install `uv` and `git`.
- Prepare a working local Daat Locus config. The default config directory is
  `~/.daat-locus`.
- Make sure the current Daat Locus repository is readable by `git`. The adapter
  packages source files with `git ls-files -co --exclude-standard`, so it
  includes local uncommitted changes as long as they are not ignored by
  `.gitignore`.

On the first run, if `-p/--path` is not provided, the runner clones DeepSWE into
`benchmark/DeepSWE/.cache/deep-swe`. Each sandbox receives a source package of
the local Daat Locus checkout, installs the required build tools inside the
container, and runs `cargo build --release --locked`.

### Run

```bash
# Change into this benchmark directory first if needed.
cd benchmark/DeepSWE

# Run the benchmark. This reuses local config, but execution happens in
# isolated Docker containers.
uv run deepswe-daat-locus
```

# parquet-merge

A small Rust CLI that merges multiple Parquet files into one output Parquet file.

## What it does

- Reads input Parquet files in the order provided.
- Uses the schema from the first file as the expected schema.
- Fails fast if any later file has a different schema.
- Writes the merged output with configurable compression.

## Requirements

- Rust stable toolchain

## Build

```bash
cargo build --release
```

Binary output:

- `target/release/parquet-merge` (Linux/macOS)
- `target\release\parquet-merge.exe` (Windows)

## Usage

```bash
parquet-merge --output merged.parquet input1.parquet input2.parquet input3.parquet
```

Compression options:

- `zstd` (default)
- `snappy`
- `gzip`
- `lz4`
- `none`

Example:

```bash
parquet-merge --output merged.parquet --compression snappy part-1.parquet part-2.parquet
```

Show the installed version:

```bash
parquet-merge --version
```

Version output behavior:

- If the build is from an exact Git tag and the repo is clean, the tag is shown.
- Otherwise, version includes commit and state, for example: `0.1.0+abc1234.clean` or `0.1.0+abc1234.dirty`.

## Behavior and guarantees

- Inputs are appended in the exact order passed on the command line.
- Output schema is exactly the schema of the first input file.
- If an input cannot be opened or read, the command returns an error.
- If schema mismatches are detected, the command returns an error and stops.

## Development

Run local checks:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --release
```

## CI and release automation

- PR checks workflow: format, clippy, tests, and release build.
- Release workflow: builds and publishes binaries for:
  - Linux x64 and aarch64
  - macOS aarch64
  - Windows x64 and aarch64
- Dependabot is configured for Cargo and GitHub Actions updates.

## Documentation

See the docs directory for implementation details and maintenance notes:

- [docs/architecture.md](docs/architecture.md)
- [docs/testing.md](docs/testing.md)
- [docs/ci-cd.md](docs/ci-cd.md)

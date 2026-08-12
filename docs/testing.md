# Testing Guide

This project has unit tests embedded in `src/main.rs` under `#[cfg(test)]`.

## What is currently tested

- Compression mapping:
  - `CompressionArg::None` maps to `Compression::UNCOMPRESSED`.

- Successful merge path:
  - Two Parquet files with identical schema are merged.
  - Output row count is validated by reading the merged file.

- Failure path:
  - A schema mismatch across input files returns an explicit error.

## Running tests

```bash
cargo test --locked
```

## Suggested future tests

- CLI-level tests for argument handling and help text.
- Test each compression mode for successful output writing.
- Large-file performance smoke test.
- Edge cases:
  - single input file merge
  - empty row groups
  - null-containing columns

## Test design notes

Tests create temporary files under the OS temp directory, write small Parquet fixtures, run merge logic, assert behavior, then clean up directories.

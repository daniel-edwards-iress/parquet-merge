# Architecture

This document explains how `parquet-merge` is structured and why.

## High-level flow

1. Parse CLI arguments with `clap`.
2. Map the requested compression flag to `parquet::basic::Compression`.
3. Open the first input file and derive the canonical schema.
4. Create an `ArrowWriter` for the output file.
5. For each input file:
   - open file
   - create a record-batch reader
   - verify schema equality with the canonical schema
   - stream each batch into the output writer
6. Close the writer and return Parquet metadata.

## Main functions

- `main()`
  - Handles CLI parsing and prints merge summary.

- `compression_from_arg(compression: CompressionArg) -> Compression`
  - Maps the CLI enum to Parquet compression settings.

- `merge_parquet_files(inputs, output, compression) -> Result<ParquetMetaData>`
  - Core merge logic used by both runtime and tests.

## Error handling

The project uses `anyhow` with contextual messages.

Common failure points:

- Missing or unreadable input file.
- Output path creation failures.
- Parquet decoding errors.
- Schema mismatch between input files.

Schema mismatch includes both expected and actual schemas in the error message.

## Data correctness notes

- Rows are preserved in input order and batch order.
- No schema evolution/unioning is attempted; exact match is required.
- Compression affects output encoding only and not logical content.

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, GzipLevel, ZstdLevel};
use parquet::file::metadata::ParquetMetaData;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, ValueEnum)]
enum CompressionArg {
    Zstd,
    Snappy,
    Gzip,
    Lz4,
    None,
}

#[derive(Parser)]
#[command(
    version = env!("PARQUET_MERGE_VERSION"),
    about = "Merge multiple parquet files into a single file"
)]
struct Cli {
    /// Input parquet files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Output parquet file
    #[arg(short, long)]
    output: PathBuf,

    /// Compression codec
    #[arg(short, long, default_value = "zstd")]
    compression: CompressionArg,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let compression = compression_from_arg(cli.compression);

    let meta = merge_parquet_files(&cli.inputs, &cli.output, compression)?;
    eprintln!(
        "Written {:?} — {} row groups, {} rows",
        cli.output,
        meta.row_groups().len(),
        meta.row_groups()
            .iter()
            .map(|rg| rg.num_rows())
            .sum::<i64>()
    );

    Ok(())
}

fn compression_from_arg(compression: CompressionArg) -> Compression {
    match compression {
        CompressionArg::Zstd => Compression::ZSTD(ZstdLevel::default()),
        CompressionArg::Snappy => Compression::SNAPPY,
        CompressionArg::Gzip => Compression::GZIP(GzipLevel::default()),
        CompressionArg::Lz4 => Compression::LZ4,
        CompressionArg::None => Compression::UNCOMPRESSED,
    }
}

fn merge_parquet_files(
    inputs: &[PathBuf],
    output: &PathBuf,
    compression: Compression,
) -> Result<ParquetMetaData> {
    if inputs.is_empty() {
        bail!("At least one input file is required");
    }

    // Derive schema from the first file
    let first_file =
        File::open(&inputs[0]).with_context(|| format!("Failed to open {:?}", inputs[0]))?;
    let schema = Arc::clone(ParquetRecordBatchReaderBuilder::try_new(first_file)?.schema());

    let props = WriterProperties::builder()
        .set_compression(compression)
        .build();

    let output_file =
        File::create(output).with_context(|| format!("Failed to create {:?}", output))?;
    let mut writer = ArrowWriter::try_new(output_file, Arc::clone(&schema), Some(props))?;

    for path in inputs {
        let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;

        if builder.schema().as_ref() != schema.as_ref() {
            bail!(
                "Schema mismatch in {:?}:\n  expected: {:?}\n  got:      {:?}",
                path,
                schema,
                builder.schema()
            );
        }

        for batch in builder.build()? {
            writer
                .write(&batch.with_context(|| format!("Failed to read batch from {:?}", path))?)?;
        }
        eprintln!("Merged: {}", path.display());
    }

    writer.close().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::{compression_from_arg, merge_parquet_files, CompressionArg};
    use clap::error::ErrorKind;
    use clap::Parser;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use parquet::arrow::ArrowWriter;
    use parquet::basic::Compression;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

    use arrow_array::{Int32Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};

    fn make_temp_dir() -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "parquet-merge-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be valid")
                .as_nanos()
        );
        path.push(unique);
        std::fs::create_dir_all(&path).expect("temp directory should be created");
        path
    }

    fn write_int_parquet(path: &Path, values: &[i32]) {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int32,
            false,
        )]));
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(Int32Array::from(values.to_vec()))],
        )
        .expect("record batch should be valid");

        let file = File::create(path).expect("input parquet should be created");
        let mut writer =
            ArrowWriter::try_new(file, schema, None).expect("writer should be created");
        writer.write(&batch).expect("batch should be written");
        writer.close().expect("writer should close");
    }

    fn write_string_parquet(path: &Path, values: &[&str]) {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Utf8,
            false,
        )]));
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![Arc::new(StringArray::from(values.to_vec()))],
        )
        .expect("record batch should be valid");

        let file = File::create(path).expect("input parquet should be created");
        let mut writer =
            ArrowWriter::try_new(file, schema, None).expect("writer should be created");
        writer.write(&batch).expect("batch should be written");
        writer.close().expect("writer should close");
    }

    #[test]
    fn maps_none_to_uncompressed() {
        assert!(matches!(
            compression_from_arg(CompressionArg::None),
            Compression::UNCOMPRESSED
        ));
    }

    #[test]
    fn supports_version_flag() {
        match super::Cli::try_parse_from(["parquet-merge", "--version"]) {
            Ok(_) => panic!("--version should trigger a display message"),
            Err(err) => assert_eq!(err.kind(), ErrorKind::DisplayVersion),
        }
    }

    #[test]
    fn merges_files_with_same_schema() {
        let dir = make_temp_dir();
        let input1 = dir.join("input1.parquet");
        let input2 = dir.join("input2.parquet");
        let output = dir.join("merged.parquet");

        write_int_parquet(&input1, &[1, 2]);
        write_int_parquet(&input2, &[3, 4]);

        let meta = merge_parquet_files(
            &[input1.clone(), input2.clone()],
            &output,
            Compression::UNCOMPRESSED,
        )
        .expect("merge should succeed");
        assert_eq!(
            meta.row_groups()
                .iter()
                .map(|rg| rg.num_rows())
                .sum::<i64>(),
            4
        );

        let file = File::open(output).expect("merged output should exist");
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .expect("reader should open")
            .build()
            .expect("reader should build");
        let total_rows = reader
            .map(|batch| batch.expect("batch should load").num_rows())
            .sum::<usize>();
        assert_eq!(total_rows, 4);

        std::fs::remove_dir_all(dir).expect("temp directory should be removed");
    }

    #[test]
    fn fails_when_schema_differs() {
        let dir = make_temp_dir();
        let input1 = dir.join("input1.parquet");
        let input2 = dir.join("input2.parquet");
        let output = dir.join("merged.parquet");

        write_int_parquet(&input1, &[1, 2]);
        write_string_parquet(&input2, &["a", "b"]);

        let err = merge_parquet_files(&[input1, input2], &output, Compression::UNCOMPRESSED)
            .expect_err("merge should fail on schema mismatch");

        assert!(
            err.to_string().contains("Schema mismatch"),
            "error should explain schema mismatch"
        );

        std::fs::remove_dir_all(dir).expect("temp directory should be removed");
    }
}

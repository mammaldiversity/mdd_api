use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

use mdd_api::mdd::diff::AllMddDiffs;
use mdd_api::parser::diff::DiffParser;
use zip::ZipWriter;
use zip::write::FileOptions;

fn create_diff_zip(path: &std::path::Path) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<()> =
        FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("MDD/MDD_v2.3.csv", options).unwrap();
    zip.write_all(&fs::read("tests/data/test_data.csv").unwrap())
        .unwrap();
    zip.start_file("MDD/Species_Syn_v2.3.csv", options).unwrap();
    zip.write_all(&fs::read("tests/data/syndata.csv").unwrap())
        .unwrap();
    zip.start_file("MDD/Diff_v2.2-v2.3.csv", options).unwrap();
    zip.write_all(&fs::read("data/Diff_v2.2-v2.3.csv").unwrap())
        .unwrap();
    zip.start_file("MDD/Diff-AllChanges_v2.2-2.3.csv", options)
        .unwrap();
    zip.write_all(&fs::read("data/Diff-AllChanges_v2.2-2.3.csv").unwrap())
        .unwrap();
    zip.start_file("release.toml", options).unwrap();
    zip.write_all(&fs::read("tests/data/release.toml").unwrap())
        .unwrap();
    zip.finish().unwrap();
}

#[test]
fn parses_diff_samples_and_writes_gzip_and_plain_json() {
    let temp = tempfile::Builder::new()
        .prefix("diff_parser")
        .tempdir()
        .unwrap();
    let diff = DiffParser::parse_files(
        std::path::Path::new("data/Diff_v2.2-v2.3.csv"),
        std::path::Path::new("data/Diff-AllChanges_v2.2-2.3.csv"),
        Some("2026-08-06".to_string()),
    )
    .unwrap();

    assert_eq!(diff.prev_version, "2.2");
    assert_eq!(diff.version, "2.3");
    assert!(!diff.taxonomy_changes.is_empty());
    assert!(!diff.all_changes.is_empty());

    let output = temp.path().join("diffs.json.gz");
    let outputs = DiffParser::write_diff(diff, &output, None, true).unwrap();
    assert_eq!(outputs.len(), 2);
    assert!(temp.path().join("diffs.json.gz").exists());
    assert!(temp.path().join("diffs.json").exists());

    let plain_json = fs::read_to_string(temp.path().join("diffs.json")).unwrap();
    assert!(plain_json.contains("\"prevVersion\""));
    assert!(plain_json.contains("\"taxonomyChanges\""));
    assert!(plain_json.contains("\"category\""));

    let loaded = AllMddDiffs::from_path(&temp.path().join("diffs.json.gz")).unwrap();
    assert_eq!(loaded.data.len(), 1);
    assert_eq!(loaded.data[0].release_date.as_deref(), Some("2026-08-06"));
    assert!(loaded.data[0].release_notes.is_none());
    assert_eq!(loaded.data[0].taxonomy_changes[0].catagory, "de novo");
}

#[test]
fn diff_cli_defaults_to_gzip_output() {
    let temp = tempfile::Builder::new()
        .prefix("diff_cli")
        .tempdir()
        .unwrap();
    let output = temp.path().join("release-diffs");
    let status = Command::new(env!("CARGO_BIN_EXE_mdd"))
        .args([
            "diff",
            "--input",
            "data/Diff_v2.2-v2.3.csv",
            "--all-changes",
            "data/Diff-AllChanges_v2.2-2.3.csv",
            "--output",
            output.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(temp.path().join("release-diffs.json.gz").exists());
}

#[test]
fn appends_gzip_diffs_and_replaces_matching_version() {
    let temp = tempfile::Builder::new()
        .prefix("diff_append")
        .tempdir()
        .unwrap();
    let first = DiffParser::parse_files(
        std::path::Path::new("data/Diff_v2.0-v2.1.csv"),
        std::path::Path::new("data/Diff-AllChanges_v2.0-v2.1.csv"),
        None,
    )
    .unwrap();
    let second = DiffParser::parse_files(
        std::path::Path::new("data/Diff_v2.1-v2.2.csv"),
        std::path::Path::new("data/Diff-AllChanges_v2.1-v2.2.csv"),
        None,
    )
    .unwrap();

    let output = temp.path().join("diffs.json.gz");
    DiffParser::write_diff(first, &output, None, false).unwrap();
    DiffParser::write_diff(second, &output, Some(&output), false).unwrap();

    let loaded = AllMddDiffs::from_file(output.to_str().unwrap()).unwrap();
    assert_eq!(loaded.data.len(), 2);
    assert_eq!(loaded.data[0].version, "2.1");
    assert_eq!(loaded.data[1].version, "2.2");

    let replacement = DiffParser::parse_files(
        std::path::Path::new("data/Diff_v2.1-v2.2.csv"),
        std::path::Path::new("data/Diff-AllChanges_v2.1-v2.2.csv"),
        Some("2026-08-06".to_string()),
    )
    .unwrap();
    DiffParser::write_diff(replacement, &output, Some(&output), false).unwrap();
    let replaced = AllMddDiffs::from_path(&output).unwrap();
    assert_eq!(replaced.data.len(), 2);
    assert_eq!(replaced.data[1].release_date.as_deref(), Some("2026-08-06"));
}

#[test]
fn unpack_exports_archive_diff_and_reads_release_date() {
    let temp = tempfile::Builder::new()
        .prefix("diff_unpack")
        .tempdir()
        .unwrap();
    let archive = temp.path().join("MDD.zip");
    let output = temp.path().join("out");
    create_diff_zip(&archive);

    let status = Command::new(env!("CARGO_BIN_EXE_mdd"))
        .args([
            "unpack",
            "--input",
            archive.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let diffs = AllMddDiffs::from_path(&output.join("diffs.json.gz")).unwrap();
    assert_eq!(diffs.data.len(), 1);
    assert_eq!(diffs.data[0].version, "2.3");
    assert_eq!(diffs.data[0].release_date.as_deref(), Some("2024-06-01"));
    assert_eq!(
        diffs.data[0].release_notes.as_deref(),
        Some("This is a sample release. (optional)")
    );
}

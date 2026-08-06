use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use flate2::write::GzEncoder;
use mdd_api::mdd::{ReleasedMddData, diff::AllMddDiffs};
use mdd_api::mil::prep::MilMetadata;
use tar::Builder;
use tempdir::TempDir;
use zip::ZipWriter;
use zip::write::FileOptions;

fn csv_with_inserted_columns(input_path: &str, inserted_columns: &[(&str, &str)]) -> String {
    let input = fs::read_to_string(input_path).unwrap();
    let mut reader = csv::Reader::from_reader(input.as_bytes());
    let source_headers = reader.headers().unwrap().clone();

    let mut headers = Vec::new();
    for header in &source_headers {
        headers.push(header.to_string());
        for (after, inserted) in inserted_columns {
            if header == *after {
                headers.push((*inserted).to_string());
            }
        }
    }

    let mut output = csv::Writer::from_writer(Vec::new());
    output.write_record(&headers).unwrap();

    for record in reader.records().take(2) {
        let record = record.unwrap();
        let mut values = Vec::new();
        for (index, header) in source_headers.iter().enumerate() {
            values.push(record.get(index).unwrap_or_default().to_string());
            for (after, _) in inserted_columns {
                if header == *after {
                    values.push(String::new());
                }
            }
        }
        output.write_record(&values).unwrap();
    }

    String::from_utf8(output.into_inner().unwrap()).unwrap()
}

fn website_species_csv() -> String {
    csv_with_inserted_columns(
        "tests/data/test_data.csv",
        &[("tribe", "subtribe"), ("nominalNames", "subspecies")],
    )
}

fn website_synonym_csv() -> String {
    let input = fs::read_to_string("tests/data/syndata.csv").unwrap();
    let mut reader = csv::Reader::from_reader(input.as_bytes());
    let source_headers = reader.headers().unwrap().clone();

    let mut headers = Vec::new();
    for header in &source_headers {
        headers.push(header.to_string());
        if header == "MDD_original_combination" {
            headers.push("MDD_normalized_original_combination".to_string());
        }
    }

    let mut output = csv::Writer::from_writer(Vec::new());
    output.write_record(&headers).unwrap();

    for (row_index, record) in reader.records().take(2).enumerate() {
        let record = record.unwrap();
        let mut values = Vec::new();
        for (index, header) in source_headers.iter().enumerate() {
            let mut value = record.get(index).unwrap_or_default().to_string();
            if header == "MDD_species_id" {
                value = if row_index == 0 {
                    "1001076".to_string()
                } else {
                    String::new()
                };
            }
            values.push(value.clone());
            if header == "MDD_original_combination" {
                values.push(value);
            }
        }
        output.write_record(&values).unwrap();
    }

    String::from_utf8(output.into_inner().unwrap()).unwrap()
}

fn website_release_toml() -> &'static str {
    r#"[metadata]
name = "The Mammal Diversity Database"
version = "v2.5"
release_date = "2026-07-28"
mdd_file = "MDD_v2.5_6904species.csv"
synonym_file = "Species_Syn_v2.5.csv"
zenodo_citation = "Mammal Diversity Database. (2026). Mammal Diversity Database (Version 2.5)."
remarks = "This is an incremental release with 6,904 total species."
"#
}

fn write_zip_entry(zip: &mut ZipWriter<File>, name: &str, contents: &[u8]) {
    let options: FileOptions<()> =
        FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file(name, options).unwrap();
    zip.write_all(contents).unwrap();
}

fn create_website_mdd_zip(path: &Path) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);

    write_zip_entry(
        &mut zip,
        "MDD/MDD_v2.5_6904species.csv",
        website_species_csv().as_bytes(),
    );
    write_zip_entry(
        &mut zip,
        "MDD/Species_Syn_Current_v2.5.csv",
        website_synonym_csv().as_bytes(),
    );
    write_zip_entry(
        &mut zip,
        "MDD/Diff_v2.4-v2.5.csv",
        b"old_name,new_name,comments,category,reference\nOld_name,New_name,Updated,changed,Reference\n",
    );
    write_zip_entry(
        &mut zip,
        "MDD/Diff-AllChanges_v2.4-v2.5.csv",
        b"species_id,old_name,new_name,category,column,old_value,new_value\n1001076,Old_name,New_name,changed,sciName,Old,New\n",
    );
    write_zip_entry(
        &mut zip,
        "MDD/META_v2.5.csv",
        b"key,value\nignored,metadata\n",
    );
    write_zip_entry(
        &mut zip,
        "MDD/TypeSpecimenMetadata_v2.5.csv",
        b"id,value\nignored,metadata\n",
    );
    write_zip_entry(
        &mut zip,
        "MDD/release.toml",
        website_release_toml().as_bytes(),
    );
    zip.finish().unwrap();
}

fn landscape_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0xF1,
        0xFF, 0x6F, 0xD3,
    ]
}

fn portrait_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x08, 0x02, 0x00, 0x00, 0x00, 0xCF,
        0xD3, 0x7E, 0x22,
    ]
}

fn create_website_mil_tar_gz(path: &Path) {
    let file = File::create(path).unwrap();
    let encoder = GzEncoder::new(file, flate2::Compression::default());
    let mut tar = Builder::new(encoder);
    let metadata = concat!(
        "Order,Family,Common Name of Family,Genus,Specific Epithet,Common name of Species,",
        "Distribution of Species,MIL #,Description of Image,Date Image Taken,Photographer,",
        "Location Where Image Taken,Original File Name\n",
        "Lagomorpha,Leporidae,Hares,Bunolagus,monticularis,Riverine Rabbit,Africa,MIL1001,",
        "A rabbit,2026-01-01,Jane Doe,South Africa,MIL1001.webp\n",
        "Lagomorpha,Leporidae,Hares,Caprolagus,hispidus?,Hispid Hare,Asia,MIL1002,",
        "An uncertain rabbit,2026-02-02,John Doe,India,MIL1002.webp\n"
    );

    let mut header = tar::Header::new_gnu();
    header.set_size(metadata.len() as u64);
    header.set_mode(0o644);
    tar.append_data(
        &mut header,
        "mil-v2026-07-28/metadata/mil_meta.csv",
        metadata.as_bytes(),
    )
    .unwrap();

    let landscape = landscape_png();
    let mut header = tar::Header::new_gnu();
    header.set_size(landscape.len() as u64);
    header.set_mode(0o644);
    tar.append_data(
        &mut header,
        "mil-v2026-07-28/images-540px-webp/MIL1001.webp",
        landscape.as_slice(),
    )
    .unwrap();

    let portrait = portrait_png();
    let mut header = tar::Header::new_gnu();
    header.set_size(portrait.len() as u64);
    header.set_mode(0o644);
    tar.append_data(
        &mut header,
        "mil-v2026-07-28/images-540px-webp/MIL1002.webp",
        portrait.as_slice(),
    )
    .unwrap();

    tar.finish().unwrap();
}

#[test]
fn unpack_handles_current_website_archive_layout() {
    let temp = TempDir::new("website_mdd_unpack").unwrap();
    let archive = temp.path().join("MDD.zip");
    let output = temp.path().join("out");
    create_website_mdd_zip(&archive);

    let status = Command::new(env!("CARGO_BIN_EXE_mdd"))
        .args([
            "unpack",
            "--input",
            archive.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--plain-text",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let json = fs::read_to_string(output.join("mdd.json")).unwrap();
    let released = ReleasedMddData::from_json(&json);
    assert_eq!(released.metadata.version, "v2.5");
    assert_eq!(released.metadata.release_date, "2026-07-28");
    assert_eq!(
        released.metadata.remarks.as_deref(),
        Some("This is an incremental release with 6,904 total species.")
    );
    assert_eq!(released.data.len(), 2);
    assert_eq!(released.synonym_only.len(), 1);
    assert_eq!(released.metadata.synonym_count, 2);

    let compressed = fs::read(output.join("mdd.json.gz")).unwrap();
    let compressed_released = ReleasedMddData::from_gz_bytes(&compressed);
    assert_eq!(compressed_released.metadata.version, "v2.5");

    for file in [
        "country_stats.json",
        "country_region_code.json",
        "usa_states.json",
        "diffs.json",
        "diffs.json.gz",
    ] {
        assert!(output.join(file).exists(), "missing generated file: {file}");
    }

    let diffs = AllMddDiffs::from_path(&output.join("diffs.json.gz")).unwrap();
    assert_eq!(diffs.data.len(), 1);
    assert_eq!(diffs.data[0].prev_version, "2.4");
    assert_eq!(diffs.data[0].version, "2.5");
    assert_eq!(diffs.data[0].release_date.as_deref(), Some("2026-07-28"));
    assert_eq!(
        diffs.data[0].release_notes.as_deref(),
        Some("This is an incremental release with 6,904 total species.")
    );
}

#[test]
fn prepare_handles_current_website_archive_and_mil_release() {
    let temp = TempDir::new("website_mdd_prepare").unwrap();
    let archive = temp.path().join("MDD.zip");
    let mil_archive = temp.path().join("mil-v2026-07-28.tar.gz");
    let output = temp.path().join("out");
    create_website_mdd_zip(&archive);
    create_website_mil_tar_gz(&mil_archive);

    let status = Command::new(env!("CARGO_BIN_EXE_mdd"))
        .args([
            "prepare",
            "--mdd-zip",
            archive.to_str().unwrap(),
            "--mil-file",
            mil_archive.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let content = fs::read_to_string(output.join("mil.json")).unwrap();
    let records: Vec<MilMetadata> = serde_json::from_str(&content).unwrap();
    assert_eq!(records.len(), 2);

    let first = records
        .iter()
        .find(|record| record.mil_id == "MIL1001")
        .unwrap();
    assert_eq!(first.mdd_id, Some(1001076));
    assert_eq!(first.orientation.as_deref(), Some("landscape"));
    assert!(!first.is_uncertain_identification);

    let second = records
        .iter()
        .find(|record| record.mil_id == "MIL1002")
        .unwrap();
    assert_eq!(second.mdd_id, Some(1001077));
    assert_eq!(second.orientation.as_deref(), Some("portrait"));
    assert!(second.is_uncertain_identification);
    assert!(output.join("mdd.json").exists());
}

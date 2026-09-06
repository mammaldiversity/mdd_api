use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use flate2::write::GzEncoder;
use tar::Builder;
use zip::ZipWriter;
use zip::write::FileOptions;

use mdd_api::mil::prep::MilMetadata;

/// Creates a mock MIL metadata CSV content.
fn get_mock_mil_csv() -> &'static str {
    "Order,Family,Common Name of Family,Genus,Specific Epithet,Common name of Species,Distribution of Species,MIL #,Description of Image,Date Image Taken,Photographer,Location Where Image Taken,Original File Name\n\
    Rodentia,Muridae,Mice,Mus,musculus,House Mouse,Cosmopolitan,MIL1001,A mouse,2026-01-01,John Doe,USA,file1.png\n\
    Rodentia,Muridae,Mice,Mus,caroli?,Ryukyu Mouse,Asia,MIL1002,Ryukyu mouse,2026-02-02,Jane Doe,Japan,file2.png\n"
}

/// Creates a mock MDD species CSV content.
fn get_mock_mdd_csv() -> &'static str {
    "id,sciName,mainCommonName,otherCommonNames,phylosort,subclass,infraclass,magnorder,superorder,order,suborder,infraorder,parvorder,superfamily,family,subfamily,tribe,genus,subgenus,specificEpithet,authoritySpeciesAuthor,authoritySpeciesYear,authorityParentheses,originalNameCombination,authoritySpeciesCitation,authoritySpeciesLink,typeVoucher,typeKind,typeVoucherURIs,typeLocality,typeLocalityLatitude,typeLocalityLongitude,nominalNames,taxonomyNotes,taxonomyNotesCitation,distributionNotes,distributionNotesCitation,subregionDistribution,countryDistribution,continentDistribution,biogeographicRealm,iucnStatus,extinct,domestic,flagged,CMW_sciName,diffSinceCMW,MSW3_matchtype,MSW3_sciName,diffSinceMSW3\n\
    100,Mus musculus,,,1,,,,,,,,,,,,,Mus,,musculus,,0,0,,,,,,,,,,,,,,,,,,,,0,0,0,,0,,,\n\
    200,Mus caroli,,,2,,,,,,,,,,,,,Mus,,caroli,,0,0,,,,,,,,,,,,,,,,,,,,0,0,0,,0,,,\n"
}

/// Creates a mock MDD synonym CSV content.
fn get_mock_syn_csv() -> &'static str {
    "MDD_syn_id,hesp_id,species_id,species,root_name,author,year,authority_parentheses,nomenclature_status,validity,original_combination,original_rank,authority_citation,unchecked_authority_citation,sourced_unverified_citations,citation_group,citation_kind,authority_page,authority_link,authority_page_link,unchecked_authority_page_link,old_type_locality,original_type_locality,unchecked_type_locality,emended_type_locality,type_latitude,type_longitude,type_country,type_subregion,type_subregion2,holotype,type_kind,type_specimen_link,order,family,genus,specific_epithet,subspecific_epithet,variant_of,senior_homonym,variant_name_citations,name_usages,comments\n\
    1,0,1,Mus musculus,Mus musculus,Linnaeus,1758,0,,valid,,species,citation,,,,,,link,,,loc,loc2,,loc3,0,0,Country,Sub,Sub2,Holotype,Kind,SpecLink,Rodentia,Muridae,Mus,musculus,,,,,,\n"
}

/// Creates a 2x1 landscape PNG dummy file.
fn get_landscape_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, // width = 2
        0x00, 0x00, 0x00, 0x01, // height = 1
        0x08, 0x02, 0x00, 0x00, 0x00, 0xF1, 0xFF, 0x6F, 0xD3,
    ]
}

/// Creates a 1x2 portrait PNG dummy file.
fn get_portrait_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, // width = 1
        0x00, 0x00, 0x00, 0x02, // height = 2
        0x08, 0x02, 0x00, 0x00, 0x00, 0xCF, 0xD3, 0x7E, 0x22,
    ]
}

/// Programmatically builds a mock MDD zip archive.
fn create_mock_mdd_zip(path: &Path) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<()> =
        FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("MDD/MDD_v2.2.csv", options).unwrap();
    zip.write_all(get_mock_mdd_csv().as_bytes()).unwrap();

    zip.start_file("MDD/Species_Syn_v2.2.csv", options).unwrap();
    zip.write_all(get_mock_syn_csv().as_bytes()).unwrap();

    zip.finish().unwrap();
}

/// Programmatically builds a mock compressed MIL tar.gz archive.
fn create_mock_mil_tar_gz(path: &Path) {
    let file = File::create(path).unwrap();
    let enc = GzEncoder::new(file, flate2::Compression::default());
    let mut tar = Builder::new(enc);

    // Add metadata CSV
    let mil_csv = get_mock_mil_csv();
    let mut header = tar::Header::new_gnu();
    header.set_size(mil_csv.len() as u64);
    header.set_mode(0o644);
    tar.append_data(&mut header, "metadata/mil_meta.csv", mil_csv.as_bytes())
        .unwrap();

    // Add MIL1001 image (landscape)
    let img1 = get_landscape_png();
    let mut header = tar::Header::new_gnu();
    header.set_size(img1.len() as u64);
    header.set_mode(0o644);
    tar.append_data(
        &mut header,
        "images-540px-webp/MIL1001.png",
        img1.as_slice(),
    )
    .unwrap();

    // Add MIL1002 image (portrait)
    let img2 = get_portrait_png();
    let mut header = tar::Header::new_gnu();
    header.set_size(img2.len() as u64);
    header.set_mode(0o644);
    tar.append_data(
        &mut header,
        "images-540px-webp/MIL1002.png",
        img2.as_slice(),
    )
    .unwrap();

    tar.finish().unwrap();
}

#[test]
fn test_cli_mil_subcommand() {
    let tmp = tempfile::Builder::new()
        .prefix("test_cli_mil")
        .tempdir()
        .unwrap();

    // Create MIL metadata file
    let mil_path = tmp.path().join("mil.csv");
    fs::write(&mil_path, get_mock_mil_csv()).unwrap();

    // Create MDD metadata file
    let mdd_path = tmp.path().join("mdd.csv");
    fs::write(&mdd_path, get_mock_mdd_csv()).unwrap();

    // Create image directory with images
    let img_dir = tmp.path().join("images");
    fs::create_dir(&img_dir).unwrap();
    fs::write(img_dir.join("MIL1001.png"), get_landscape_png()).unwrap();
    fs::write(img_dir.join("MIL1002.png"), get_portrait_png()).unwrap();

    // Output path
    let output_json = tmp.path().join("output.json");

    // Run the subcommand 'mil' using cargo run
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("mil")
        .arg("-m")
        .arg(&mil_path)
        .arg("-d")
        .arg(&mdd_path)
        .arg("-i")
        .arg(&img_dir)
        .arg("-o")
        .arg(&output_json)
        .status()
        .expect("Failed to execute cargo run mil");

    assert!(status.success(), "CLI command 'mil' failed");

    // Verify output file content
    let content = fs::read_to_string(&output_json).expect("Failed to read output JSON");
    let records: Vec<MilMetadata> =
        serde_json::from_str(&content).expect("Failed to deserialize output JSON");

    assert_eq!(records.len(), 2);

    let r1 = records.iter().find(|r| r.mil_id == "MIL1001").unwrap();
    assert_eq!(r1.mdd_id, Some(100));
    assert_eq!(r1.orientation.as_deref(), Some("landscape"));
    assert_eq!(r1.is_uncertain_identification, false);

    let r2 = records.iter().find(|r| r.mil_id == "MIL1002").unwrap();
    assert_eq!(r2.mdd_id, Some(200));
    assert_eq!(r2.orientation.as_deref(), Some("portrait"));
    assert_eq!(r2.is_uncertain_identification, true);
}

#[test]
fn test_cli_prepare_subcommand_compressed() {
    let tmp = tempfile::Builder::new()
        .prefix("test_cli_prepare")
        .tempdir()
        .unwrap();

    // Create mock MDD zip file
    let mdd_zip_path = tmp.path().join("MDD.zip");
    create_mock_mdd_zip(&mdd_zip_path);

    // Create mock MIL compressed tar.gz file
    let mil_tar_gz_path = tmp.path().join("mil.tar.gz");
    create_mock_mil_tar_gz(&mil_tar_gz_path);

    // Output directory
    let output_dir = tmp.path().join("out");

    // Run the subcommand 'prepare' using cargo run
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("prepare")
        .arg("-z")
        .arg(&mdd_zip_path)
        .arg("-m")
        .arg(&mil_tar_gz_path)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .expect("Failed to execute cargo run prepare");

    assert!(status.success(), "CLI command 'prepare' failed");

    // Verify prepared MIL-MDD JSON file is created
    let prepared_json_path = output_dir.join("mil.json");
    assert!(
        prepared_json_path.exists(),
        "Prepared MIL JSON does not exist"
    );

    let content = fs::read_to_string(&prepared_json_path).expect("Failed to read prepared JSON");
    let records: Vec<MilMetadata> =
        serde_json::from_str(&content).expect("Failed to deserialize prepared JSON");

    assert_eq!(records.len(), 2);

    let r1 = records.iter().find(|r| r.mil_id == "MIL1001").unwrap();
    assert_eq!(r1.mdd_id, Some(100));
    assert_eq!(r1.orientation.as_deref(), Some("landscape"));

    let r2 = records.iter().find(|r| r.mil_id == "MIL1002").unwrap();
    assert_eq!(r2.mdd_id, Some(200));
    assert_eq!(r2.orientation.as_deref(), Some("portrait"));
}

use mdd_api::parser::metadata::ReleaseToml;
use std::path::Path;

#[test]
fn test_from_file() {
    let release_meta = Path::new("tests/data/release.toml");

    let metadata = ReleaseToml::from_file(&release_meta).unwrap();
    let toml_content = std::fs::read_to_string(&release_meta).unwrap();
    let expected_metadata = ReleaseToml::from_toml(toml_content.as_str()).unwrap();
    assert_eq!(metadata.to_toml(), expected_metadata.to_toml());
    assert_eq!(metadata.metadata.name, "MDD");
    assert_eq!(metadata.metadata.version, "2.2.1");
    assert_eq!(metadata.metadata.release_date.as_str(), "2024-06-01");
    assert_eq!(metadata.metadata.mdd_file.as_str(), "mdd_2024_1.csv");
    assert_eq!(
        metadata.metadata.synonym_file.as_str(),
        "synonyms_2024_1.csv"
    );
    assert_eq!(
        metadata.metadata.doi.as_deref(),
        Some("https://doi.org/10.5281/zenodo.17033774")
    );
    assert_eq!(
        metadata.metadata.remarks.as_deref(),
        Some("This is a sample release. (optional)")
    );
}

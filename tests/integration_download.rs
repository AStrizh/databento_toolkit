use databento_toolkit::download_history;
use time::macros::date;
use std::fs;

#[tokio::test]
async fn test_download_history_creates_structure() {
    let base_path = "test_output";

    let result = download_history(
        date!(2022 - 10 - 01),
        date!(2022 - 12 - 31),
        &["CL", "NG", "ES", "NQ"],
        base_path,
    ).await;

    assert!(result.is_ok());

    // Check that at least one directory was created
    // let paths = fs::read_dir(base_path).unwrap();
    for symbol in ["CL", "NG", "ES", "NQ"] {
        let mut found = false;
        for entry in fs::read_dir(base_path).unwrap() {
            let path = entry.unwrap().path();
            if path.join(symbol).exists() {
                found = true;
                break;
            }
        }
        assert!(found, "Directory for {} not found", symbol);
    }
}
use databento_toolkit::download_history;
use time::macros::date;
use std::fs;

#[tokio::test]
async fn test_download_history_creates_structure() {
    let base_path = "test_output";
    
    // Clean up before test run (optional but helps avoid false positives)
    if std::path::Path::new(base_path).exists() {
        std::fs::remove_dir_all(base_path).unwrap();
    }

    let result = download_history(
        date!(2022 - 10 - 01),
        date!(2025 - 12 - 31),
        &["CL", "NG", "ES", "NQ"],
        base_path,
    ).await;

    assert!(result.is_ok());

    for symbol in ["CL", "NG", "ES", "NQ"] {
        let symbol_dir = format!("{}/{}", base_path, symbol);
        let path = std::path::Path::new(&symbol_dir);

        // Confirm directory for symbol was created
        assert!(
            path.exists() && path.is_dir(),
            "Directory for {} not found",
            symbol
        );

        // Check for presence of at least one .mock file (mocked download)
        let entries: Vec<_> = fs::read_dir(path)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "mock")
                    .unwrap_or(false)
            })
            .collect();

        assert!(
            !entries.is_empty(),
            "No .mock files found in {} directory",
            symbol
        );
    }
}
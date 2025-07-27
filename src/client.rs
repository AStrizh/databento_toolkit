use databento::HistoricalClient;
use dotenvy::var;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Lazily initialized global singleton client, protected by a Mutex
static CLIENT: Lazy<Mutex<HistoricalClient>> = Lazy::new(|| {
    let api_key = var("DATABENTO_API_KEY")
        .expect("DATABENTO_API_KEY must be set in the environment or .env file");

    let client = HistoricalClient::builder()
        .key(&api_key)
        .expect("Failed to set API key")
        .build()
        .expect("Failed to build HistoricalClient");

    Mutex::new(client)
});

/// Get a locked reference to the global HistoricalClient.
/// Panics if the mutex is poisoned (shouldn't happen unless code panics while holding the lock).
pub fn get_client() -> std::sync::MutexGuard<'static, HistoricalClient> {
    CLIENT.lock().expect("Failed to acquire lock on HistoricalClient")
}

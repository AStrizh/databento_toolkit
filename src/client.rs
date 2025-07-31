use databento::HistoricalClient;

/// Wraps a `HistoricalClient` and hides API key management
#[derive(Clone)]
pub struct DBClient {
    client: HistoricalClient,
}

impl DBClient {
    /// Creates a new client using the API key from the environment
    pub fn new() -> Self {
        let api_key = std::env::var("DATABENTO_API_KEY")
            .expect("DATABENTO_API_KEY must be set");

        let client = HistoricalClient::builder()
            .key(&api_key).unwrap()
            .build().unwrap();

        Self { client }
    }

    pub fn get_mut(&mut self) -> &mut HistoricalClient {
        &mut self.client
    }

}

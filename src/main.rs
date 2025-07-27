use databento::{
    dbn::Schema, historical::metadata::GetCostParams,
    HistoricalClient,
};
use time::macros::datetime;


#[tokio::main]
async fn main() -> databento::Result<()> {
    let api_key = dotenvy::var("DATABENTO_API_KEY")
        .expect("DATABENTO_API_KEY must be set in the environment");

    let mut client = HistoricalClient::builder()
        .key(api_key)?
        .build()?;

    let cost = client
        .metadata()
        .get_cost(
            &GetCostParams::builder()
                .dataset("GLBX.MDP3")
                .date_time_range((
                    datetime!(2023-06-01 00:00 UTC),
                    datetime!(2023-06-30 23:59 UTC),
                ))
                .symbols("CLN3")
                .schema(Schema::Ohlcv1M)
                .build(),
        )
        .await?;
    println!("{cost:.4}");
    Ok(())
}
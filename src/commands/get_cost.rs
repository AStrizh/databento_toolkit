use crate::client::get_client;
use databento::{dbn::Schema, historical::metadata::GetCostParams};
use time::macros::datetime;

pub async fn run_get_cost() -> databento::Result<()> {
    let mut client = get_client();

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

    println!("Cost for request: ${:.4}", cost);
    Ok(())
}

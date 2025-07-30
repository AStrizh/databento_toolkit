use databento::dbn::Schema;
use databento::historical::timeseries::GetRangeToFileParams;
use time::Date;
use crate::client::get_client;

pub async fn download_data(symbol: &str, base_path: &str, start: Date, end: Date) -> databento::Result<()> {
    let mut client = get_client();
    let path = format!("{base_path}.dbn.zst");

    client
        .timeseries()
        .get_range_to_file(
            &GetRangeToFileParams::builder()
                .dataset("GLBX.MDP3")
                .date_time_range((start.midnight().assume_utc(), end.midnight().assume_utc()))
                .symbols(symbol.to_string())
                .schema(Schema::Ohlcv1M)
                .path(path)
                .build(),
        )
        .await?;

    Ok(())
}
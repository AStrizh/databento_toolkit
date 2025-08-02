# Databento Toolkit (Unofficial)
(Last commit broke stuff. Will fix tomorrow)  

This project [WILL] provide(s) tools for downloading, decoding, and processing historical market data using the [Databento API](https://docs.databento.com/) and the Rust Programming Language. 

Currently only configured to download 1-minute bar data (OHLCV) for CL (WTI Light Sweet Crude Oil) futures contracts (should be simple enough to modify for other Futures symbols). 
It specifically downloads the last 40 days of a contract (when it is front of the month). Time periods can be adjusted in the code.

- This is an independent project by Aleksandr Strizhevskiy, and is not supported by Databento

- Built with JetBrain's RustRover IDE (and ChatGPT {I promise I tested it} ) 
---



## Features

1. **Download Data**
    - Fetch historical market data from the Databento API.
    - Downloads are batched and concurrency-controlled to avoid API throttling.
    - Data is stored in a file format suitable for further decoding and processing.


2. **Customizable Contract Periods**
    - Automatically generate contract periods for CL futures (e.g., `CLN3`) based on expiration dates.
    - Configure the range of years for generation.
   

3. **Decode Data**
    - Decode Databento's `.dbn.zst` compressed binary format into JSON.


4. **Request a Quote (Get API Cost)**
    - Allows users to estimate the cost of a request for specific datasets, symbols, and date ranges before making an actual download.

---

## Prerequisites & Setup

Before using this toolkit, ensure you have the following:

1. **Rust Development Environment**
    - Install Rust from [rust-lang.org](https://www.rust-lang.org/).
    - Use version `1.88.0` or later.


2. **Databento Account**
    - Create an account at [Databento](https://www.databento.com/) and obtain an API key.
    - Copy (or rename) the example environment file `.env.example`  to `.env`.
    - Add the API key to your `.env` file.


3. **Dependencies**
   The project depends on several Rust libraries, specified in the `Cargo.toml`:
    - [`databento`](https://crates.io/crates/databento)
    - [`tokio`](https://tokio.rs/) for async runtime and concurrency.
    - [`serde`](https://serde.rs/) and [`serde_json`](https://crates.io/crates/serde_json) for deserialization.
    - [`dotenvy`](https://docs.rs/dotenvy) for loading environment variables.

---


## How to Use

### 1. **Download Historical Data**

This project was written with RustRover, so instructions will be for that IDE.

To download historical data, go into `Run/Debug Configurations` in the IDE's Run Settings and append `-- download` to Command.
The full path should look like this:

`run --package databento_toolkit --bin databento_toolkit -- download`

Alternatively, you can run the project from the terminal as such:
```shell script
cargo run --package databento_toolkit --bin databento_toolkit -- download
```

Currently, downloads crude oil CL contract data for a specified set of years.

You can customize the base folder where downloads are saved by editing the `base_path` variable in `main.rs`.

- **Default Download Folder:** `Hist_Fut_Data/`
- Each dataset is organized into subfolders based on the year.

---

### 2. **Decode Files**

After downloading, you’ll need to decode the `.dbn.zst` files into a readable format. 
Use the `decode` task to convert the downloaded data.

The decode task reads files in the `Hist_Fut_Data/` directory, decodes them, and saves them as JSON.

---

### 3. **Get a Quote (API Cost Estimation)**

The `quote` task lets you estimate the cost of a request before downloading data.  
Currently, it is hardcoded. I plan to change that in the future. Modify with values of interest to you.

By default, the functionality uses the following parameters:
- **Dataset:** `GLBX.MDP3`
- **Date Range:** `2023-06-01` to `2023-06-30`
- **Symbols:** `CLN3`
- **Schema:** `Ohlcv1M` (OHLCV with 1-minute granularity)

The printed output will show the cost estimate (in USD) for this request:
```
Cost for request: $0.0599
```

### **Note**:
It cost $3.21 to download 24 months (24 contracts 40 days each) of CL futures data in 1-minute bars (this is the code as written now).  
You may optimize it further to download even fewer bars, further reducing costs.  
Costs seem to vary from contract to contract and month to month.

---



### Code Overview

The project is organized into modular files to separate concerns:

#### 1. `client.rs`
Handles API client creation and wraps the Databento `HistoricalClient` with simplified setup using environment variables.

#### 2. `types.rs`
Defines core data structures, including:
- `DownloadTask`: Encapsulates metadata for each download operation.
- `JsonOhlcv`: A deserializable representation of OHLCV data for usage after decoding.

#### 3. `download.rs`
Main logic for downloading data:
- Generates CL contracts using CME rules for expiration dates.
- Uses asynchronous tasks to fetch and save historical data files.
- Employs a `tokio::sync::Semaphore` to limit concurrent API calls.

#### 4. `contracts.rs`
Defines CME rules for generating contract symbols (e.g., `CLN3`) and their expiration periods (only CL for now):
- Maps months of the year to futures month codes.
- Calculates download start and end dates for each contract.

#### 5. `fetch.rs`
Handles the actual download of data using the Databento API client for the generated contract periods.

#### 6. `decode.rs`
Processes `.dbn.zst` files from the download directory and decodes them into JSON.

#### 7. `get_quote.rs`
Returns the estimated cost of a history download request from Databento.

---

## Customization Notes

1. **Years of Data**
   To modify the range of years, adjust the logic in `download.rs`:
```rust
let periods = generate_cl_contract_periods(start_year, end_year);
```

Replace `start_year` and `end_year` with your desired range. (I will add ability to specify dates and hours soon)

2. **Concurrency Limits**
   Semaphore limits concurrency to avoid API throttling. Adjust this line in `download.rs`:
```rust
let semaphore = Arc::new(Semaphore::new(10)); // Change 10 to desired limit
```


3. **Folder Structure**
   By default, files are organized by year in a `base_path` folder. You can change the `base_path` variable in `main.rs` to a different root directory.

4. **Symbols**
   The project is optimized for crude oil (CL) contracts. To add support for other symbols, adapt the `generate_cl_contract_periods` function in `contracts.rs`.

---

## Example Workflow

1. **Download the Data:**
   Run the `download` task to fetch `.dbn.zst` datasets:
```shell script
cargo run --package databento_toolkit --bin databento_toolkit -- download
```


2. **Verify Files:**
   Downloaded files are saved in `Hist_Fut_Data/<year>/<symbol>.dbn.zst`. Example:
```
Hist_Fut_Data/2023/CLN3.dbn.zst
```


3. **Decode Files:**
   Use the `decode` task:
```shell script
cargo run --package databento_toolkit --bin databento_toolkit -- decode
```
Decoded files are saved in the same directory (`Hist_Fut_Data`).

---

## Future Enhancements

- **Symbol and Date Flexibility:**
  Extend support for downloading and processing other futures symbols (e.g., GC for gold or ES for S&P 500).
- **Pretty Print Option:**
    Currently, decoded times are in Unix time and prices are in integers with no decimals.
- **Other Data Sets:**
    Databento gives to option to download data from different sources in many different formats (including tick data).
    
    Low priority.
- **A Graphical User Interface:**
  Very low priority
---

## Troubleshooting

- Honestly, just ask ChatGPT

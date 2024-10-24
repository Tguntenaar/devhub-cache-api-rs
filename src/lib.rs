pub mod api_background_service;
pub mod api_client;
pub mod db;
pub mod nearblocks_client;
pub mod rpc_service;
pub mod types;
use chrono::DateTime;

pub fn timestamp_to_date_string(timestamp: i64) -> String {
    // Convert the timestamp to a NaiveDateTime
    let datetime = DateTime::from_timestamp(timestamp, 0);

    // Format the NaiveDateTime to a string in YYYY-MM-DD format
    datetime.unwrap().format("%Y-%m-%d").to_string()
}

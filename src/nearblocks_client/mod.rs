use near_sdk::AccountId;
use reqwest::Client;
use serde::{Deserialize, Serialize};
pub mod transactions;
pub mod types;
use types::Transaction;

// TODO use nearblocks API KEY

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    pub txns: Vec<Transaction>,
}

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            base_url: "https://api.nearblocks.io/".to_string(),
            client: Client::new(),
        }
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_account_txns_by_pagination(
        &self,
        account_id: AccountId,
        method: Option<String>,
        since_date: Option<String>,
        limit: Option<i32>,
        order: Option<String>,
    ) -> Result<ApiResponse, reqwest::Error> {
        let query_params = format!(
            "?method={}&after_date={}&page=1&per_page={}&order={}",
            method.unwrap_or_default(),
            since_date.unwrap_or("2024-10-10".to_string()),
            limit.unwrap_or(10),
            order.unwrap_or("desc".to_string())
        );
        let endpoint = format!("v1/account/{}/txns", account_id);
        let url = self.base_url.clone() + &endpoint + &query_params;
        let res = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;
        Ok(res)
    }
}

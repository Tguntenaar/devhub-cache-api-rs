use near_sdk::AccountId;
use reqwest::Client;
use serde::{Deserialize, Serialize};
pub mod transactions;
pub mod types;
use types::Transaction;

// TODO use nearblocks API KEY

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    #[serde(default)]
    pub txns: Vec<Transaction>,
}

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
    api_key: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            base_url: "https://api.nearblocks.io/".to_string(),
            client: Client::new(),
            api_key: "".to_string(),
        }
    }
}

impl ApiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.nearblocks.io/".to_string(),
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_account_txns_by_pagination(
        &self,
        account_id: AccountId,
        since_date: Option<String>,
        limit: Option<i32>,
        order: Option<String>,
    ) -> Result<ApiResponse, reqwest::Error> {
        let query_params = format!(
            "?after_date={}&page=1&per_page={}&order={}",
            since_date.unwrap_or("2024-10-10".to_string()),
            limit.unwrap_or(50),
            order.unwrap_or("desc".to_string())
        );
        let endpoint = format!("v1/account/{}/txns", account_id);
        let url = self.base_url.clone() + &endpoint + &query_params;
        println!("Fetching from {}", url);
        self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json::<ApiResponse>()
            .await
    }
}

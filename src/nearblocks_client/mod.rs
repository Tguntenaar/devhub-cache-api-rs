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
        after_block: Option<i64>,
        limit: Option<i32>,
        order: Option<String>,
        page: Option<i32>,
    ) -> Result<ApiResponse, reqwest::Error> {
        let query_params = format!(
            "?after_block={}&per_page={}&order={}&page={}",
            after_block.unwrap_or(0),
            limit.unwrap_or(50),
            order.unwrap_or("asc".to_string()),
            page.unwrap_or(1)
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

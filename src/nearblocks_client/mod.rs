use near_sdk::AccountId;
use reqwest::Client;
use serde::{Deserialize, Serialize};
pub mod proposal;
pub mod rfp;
pub mod transactions;
pub mod types;
use types::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    #[serde(default)]
    pub txns: Vec<Transaction>,
    pub cursor: Option<String>,
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
        cursor: String,
        limit: Option<i32>,
        order: Option<String>,
        page: Option<i32>,
    ) -> Result<ApiResponse, reqwest::Error> {
        let base_params = self.build_pagination_params(limit, order, page);
        let query_params = self.add_cursor_param(base_params, cursor);
        let endpoint = format!("v1/account/{}/txns", account_id);
        let url = self.base_url.clone() + &endpoint + &query_params;

        println!("Fetching transactions from {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        match response.json::<ApiResponse>().await {
            Ok(api_response) => {
                println!(
                    "Successfully fetched {} transactions",
                    api_response.txns.len()
                );
                Ok(api_response)
            }
            Err(e) => {
                eprintln!("Failed to parse API response: {}", e);
                Err(e)
            }
        }
    }

    fn build_pagination_params(
        &self,
        limit: Option<i32>,
        order: Option<String>,
        page: Option<i32>,
    ) -> String {
        format!(
            "?per_page={}&order={}&page={}",
            limit.unwrap_or(50),
            order.unwrap_or_else(|| "asc".to_string()),
            page.unwrap_or(1),
        )
    }

    fn add_cursor_param(&self, base_params: String, cursor: String) -> String {
        if cursor.is_empty() {
            format!("{}&after_block=0", base_params)
        } else {
            format!("{}&cursor={}", base_params, cursor)
        }
    }
}

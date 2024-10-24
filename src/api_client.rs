use reqwest::Client;
use serde::Deserialize;
use tokio::time::{self, Duration};

#[derive(Deserialize)]
pub struct ApiResponse {
    // Define the response fields
    pub data: String,
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    api_url: String,
}

// TODO Create a nearblocks client
// https://github.com/Nearblocks/nearblocks/tree/main/apps/api/src
// https://api.nearblocks.io/v1/account/devhub.near/txns?method=add_proposal&after_date=2024-10-10&page=1&per_page=25&order=desc

impl ApiClient {
    pub fn new(api_url: &str) -> Self {
        Self {
            client: Client::new(),
            api_url: api_url.to_string(),
        }
    }

    pub async fn get_data(&self) -> Result<ApiResponse, reqwest::Error> {
        let res = self
            .client
            .get(&self.api_url)
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;
        Ok(res)
    }
}

use crate::api_client::ApiClient;
use tokio::{
    signal,
    task::JoinHandle,
    time::{self, Duration},
};

pub struct ApiBackgroundService {
    api_client: ApiClient,
    handle: Option<JoinHandle<()>>,
}

impl ApiBackgroundService {
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            api_client,
            handle: None,
        }
    }

    // Spawns the background task and starts it
    pub fn start(&mut self) {
        let api_client = self.api_client.clone();
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;

                match api_client.get_data().await {
                    Ok(response) => {
                        println!("Received data: {:?}", response.data);
                    }
                    Err(e) => {
                        eprintln!("Error fetching data: {:?}", e);
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    // Handles graceful shutdown of the background task
    pub async fn shutdown(self) {
        if let Some(handle) = self.handle {
            handle.abort(); // Abort the background task
            println!("Background task aborted");
        }
    }

    // Wait for a shutdown signal (e.g., Ctrl+C)
    pub async fn wait_for_shutdown(self) {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        self.shutdown().await;
    }
}

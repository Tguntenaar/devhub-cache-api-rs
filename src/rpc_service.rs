use devhub_shared::proposal::VersionedProposal;
use near_account_id::AccountId;
use near_api::{types::Data, Contract, NetworkConfig};
use rocket::http::Status;
use rocket::serde::json::json;
use rocket::FromForm;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RpcResponse {
    pub data: String,
}

#[derive(Clone)]
pub struct RpcService {
    network: NetworkConfig,
    contract: Contract,
}

#[derive(FromForm)]
pub struct ProposalParams {
    proposal_ids: Option<Vec<i32>>,
}

/**
 * Usage
 * use devhub_cache_api::rpc_service::RpcService;
 * let rpc_service = RpcService::new(Some("devhub.near".parse::<AccountId>().unwrap()));
 * let proposals = rpc_service.get_proposals().await;
 */

impl RpcService {
    pub fn new(account_id: Option<AccountId>) -> Self {
        Self {
            network: NetworkConfig::mainnet(),
            contract: Contract(account_id.unwrap_or("devhub.near".parse::<AccountId>().unwrap())),
        }
    }

    pub async fn get_proposals(&self) -> Result<String, String> {
        // TODO: Add query params , params: ProposalParams
        let params: ProposalParams = ProposalParams {
            proposal_ids: Some(vec![200, 199]),
        };

        let mut args = json!({});
        if let Some(proposal_ids) = params.proposal_ids {
            println!("Proposal ids: {:?}", proposal_ids);
            args = json!({ "ids": proposal_ids });
        }

        let result: Result<Data<Vec<VersionedProposal>>, _> = self
            .contract
            .call_function("get_proposals", args)
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        let data_vector_proposals = result.unwrap();
        let proposals = data_vector_proposals
            .data
            .into_iter()
            .map(|proposal: VersionedProposal| proposal)
            .collect::<Vec<VersionedProposal>>();

        Ok(format!("Hello, {:?}!", proposals))
    }

    pub async fn get_all_proposal_ids(&self) -> Result<String, Status> {
        let result: Result<Data<Vec<i32>>, _> = self
            .contract
            .call_function("get_all_proposal_ids", ())
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        match result {
            Ok(current_value) => {
                println!("Current value: {:?}", current_value);
                Ok(format!("Hello, {:?}!", current_value))
            }
            Err(e) => {
                println!("Error fetching proposal ids: {:?}", e);
                Err(rocket::http::Status::InternalServerError)
            }
        }
    }
}

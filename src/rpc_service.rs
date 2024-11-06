use devhub_shared::proposal::VersionedProposal;
use devhub_shared::rfp::VersionedRFP;
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

impl Default for RpcService {
    fn default() -> Self {
        Self {
            network: NetworkConfig::mainnet(),
            contract: Contract("devhub.near".parse::<AccountId>().unwrap()),
        }
    }
}

/**
 * Usage
 * use devhub_cache_api::rpc_service::RpcService;
 * let rpc_service = RpcService::new(Some("devhub.near".parse::<AccountId>().unwrap()));
 * let proposals = rpc_service.get_proposals().await;
 */

impl RpcService {
    pub fn new(account_id: Option<AccountId>) -> Self {
        match account_id {
            Some(id) => Self {
                network: NetworkConfig::mainnet(),
                contract: Contract(id),
            },
            None => Self::default(),
        }
    }

    pub async fn get_proposal(&self, proposal_id: i32) -> Result<VersionedProposal, String> {
        let result: Result<Data<VersionedProposal>, _> = self
            .contract
            .call_function("get_proposal", json!({ "proposal_id": proposal_id }))
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        match result {
            Ok(proposal) => Ok(proposal.data),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_rfp(&self, rfp_id: i32) -> Result<VersionedRFP, String> {
        let result: Result<Data<VersionedRFP>, _> = self
            .contract
            .call_function("get_rfp", json!({ "rfp_id": rfp_id }))
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        match result {
            Ok(proposal) => Ok(proposal.data),
            Err(e) => Err(e.to_string()),
        }
    }

    // TODO return value should it be Result or Option?
    pub async fn get_proposals(&self) -> Result<Vec<VersionedProposal>, String> {
        // TODO: Add query params , params: ProposalParams
        let params: ProposalParams = ProposalParams {
            proposal_ids: Some(vec![200, 199]),
        };

        let mut args = json!({});
        if let Some(proposal_ids) = params.proposal_ids {
            println!("Proposal ids: {:?}", proposal_ids);
            args = json!({ "ids": proposal_ids });
        }

        match self
            .contract
            .call_function("get_proposals", args)
            .unwrap()
            .read_only::<Vec<VersionedProposal>>()
            .fetch_from(&self.network)
            .await
        {
            Ok(res) => Ok(res.data),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_all_proposal_ids(&self) -> Result<Vec<i32>, Status> {
        let result: Result<Data<Vec<i32>>, _> = self
            .contract
            .call_function("get_all_proposal_ids", ())
            .unwrap()
            .read_only()
            .fetch_from(&self.network)
            .await;

        match result {
            Ok(res) => Ok(res.data),
            Err(e) => {
                println!("Error fetching proposal ids: {:?}", e);
                Err(rocket::http::Status::InternalServerError)
            }
        }
    }
}

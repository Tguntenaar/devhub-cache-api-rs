use devhub_shared::proposal::VersionedProposal;
use near::{types::Data, Contract, NetworkConfig};
use near_account_id::AccountId;
use rocket::serde::json::json;
use rocket::FromForm;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RpcResponse {
    // Define the response fields
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

impl RpcService {
    pub fn new(account_id: Option<AccountId>) -> Self {
        Self {
            network: NetworkConfig::mainnet(),
            contract: Contract(account_id.unwrap_or("devhub.near".parse::<AccountId>().unwrap())),
        }
    }

    // TODO: Return Result<String, ERROR>
    // TODO: Add query params , params: ProposalParams
    pub async fn get_proposals(&self) -> Result<String, String> {
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
}

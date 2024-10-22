// use crate::db::DBTrait;
use devhub_cache_api::db::DB;
use devhub_shared::proposal::VersionedProposal;
use near::{types::Data, Contract, NetworkConfig};
use near_account_id::AccountId;
use rocket::request::FromParam;
use rocket::serde::json::json;
use rocket::serde::Serialize;
use rocket::{get, FromForm, State};

#[derive(Debug, Serialize)]
struct ProposalIds(Vec<i32>);

impl<'r> FromParam<'r> for ProposalIds {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        let ids = param
            .split(',')
            .map(|s| s.parse::<i32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Invalid integer")?;
        Ok(ProposalIds(ids))
    }
}

// #[get("/get_all_proposal_ids")]
async fn get_all_proposal_ids() -> Result<String, rocket::http::Status> {
    let mainnet = near_workspaces::mainnet()
        .await
        .map_err(|_e| rocket::http::Status::InternalServerError)?;
    let account_id = "devhub.near".parse::<AccountId>().unwrap();
    let network = NetworkConfig::from(mainnet);
    let contract = Contract(account_id);

    // Let's fetch current value on a contract
    let result: Result<Data<Vec<i32>>, _> = contract
        // Please note that you can add any argument as long as it is deserializable by serde :)
        // feel free to use serde_json::json macro as well
        .call_function("get_all_proposal_ids", ())
        .unwrap()
        .read_only()
        .fetch_from(&network)
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

#[derive(FromForm)]
struct ProposalParams {
    proposal_ids: Option<Vec<i32>>,
}

// Struct for query parameters
#[derive(Debug, FromForm)]
struct ProposalQuery {
    limit: Option<usize>, // Optional limit parameter
    sort: Option<String>, // Optional sorting parameter
}

// TODO add query params
#[utoipa::path(get, path = "/proposals")]
#[get("/")]
async fn get_proposals(db: &State<DB>) -> Result<String, rocket::http::Status> {
    let params = ProposalParams {
        proposal_ids: Some(vec![200, 199]),
    };

    if let Some(proposal_ids) = params.proposal_ids {
        println!("Proposal ids: {:?}", proposal_ids);

        let mainnet = near_workspaces::mainnet()
            .await
            .map_err(|_e| rocket::http::Status::InternalServerError)?;
        let account_id = "devhub.near".parse::<AccountId>().unwrap();
        let network = NetworkConfig::from(mainnet);
        let contract = Contract(account_id);
        println!("Proposal ids: {:?}", proposal_ids);

        // TODO
        // let proposal_ids = proposal_ids.unwrap_or(ProposalIds(vec![]));
        println!("Proposal ids: {:?}", proposal_ids);
        // Let's fetch current value on a contract
        // : Result<Data<Vec<Proposal>>, _>
        let result: Result<Data<Vec<VersionedProposal>>, _> = contract
            .call_function("get_proposals", json!({ "ids": proposal_ids }))
            .unwrap()
            .read_only()
            .fetch_from(&network)
            .await;

        // Store in postgres
        let data_vector_proposals = result.unwrap();
        let proposals = data_vector_proposals
            .data
            .into_iter()
            .map(|proposal: VersionedProposal| proposal.into())
            .collect::<Vec<VersionedProposal>>();

        let mut tx = db.begin().await.unwrap();
        // Upsert into postgres
        // proposals.clone().into_iter().for_each(|proposal| {
        //     let VersionedProposal::V0(proposal_v0) = proposal;
        //     // TODO: Upsert into postgres

        //     db.insert_proposal(&mut tx, "thomasguntenaar.near".to_string())
        //         .await
        //         .unwrap();
        // });

        Ok(format!("Hello, {:?}!", proposals))
    } else {
        Ok("No proposal ids provided".to_string())
    }
    //         Err(rocket::http::Status::InternalServerError)
}

#[utoipa::path(get, path = "/proposals/{proposal_id}")]
#[get("/<proposal_id>")]
async fn get_proposal(proposal_id: i32, db: &State<DB>) -> Result<String, rocket::http::Status> {
    Ok(format!("Hello, {:?}!", proposal_id))
}

pub fn stage() -> rocket::fairing::AdHoc {
    // rocket
    rocket::fairing::AdHoc::on_ignite("Proposal Stage", |rocket| async {
        println!("Proposal stage on ignite!");

        rocket.mount("/proposals/", rocket::routes![get_proposals, get_proposal])
    })
}

use std::collections::HashSet;

// use crate::db::DBTrait;
use devhub_cache_api::db::DB;
use devhub_cache_api::{nearblocks_client, timestamp_to_date_string};
use devhub_shared::proposal::{Proposal, VersionedProposalBody};
use near::{types::Data, Contract, NetworkConfig};
use near_account_id::AccountId;
use rocket::request::FromParam;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, http::Status, FromForm, State};

// use devhub_cache_api::rpc_service::RpcService;
// let rpc_service = RpcService::new(Some("devhub.near".parse::<AccountId>().unwrap()));
// let proposals = rpc_service.get_proposals().await;

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
async fn get_all_proposal_ids() -> Result<String, Status> {
    let mainnet = near_workspaces::mainnet()
        .await
        .map_err(|_e| Status::InternalServerError)?;
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

#[derive(Serialize, Deserialize)]
struct AddProposalArgs {
    body: VersionedProposalBody,
    labels: HashSet<String>,
    accepted_terms_and_conditions_version: Option<near_sdk::BlockHeight>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SetBlockHeightCallbackArgs {
    proposal: Proposal,
}

// add query params to get_proposals entrypoint
#[utoipa::path(get, path = "/proposals")]
#[get("/")]
async fn get_proposals(db: &State<DB>) -> Result<Json<Proposal>, Status> {
    // Get current timestamp
    let current_timestamp = chrono::Utc::now().timestamp();
    // Get last timestamp when database was updated
    let last_updated_timestamp = db.get_last_updated_timestamp().await.unwrap();

    // TODO the timestamps are way off between blockchain and database.
    println!("last_updated_timestamp: {:?}", last_updated_timestamp); // 1709470463748732291
    println!("current_timestamp: {:?}", current_timestamp); // 1729806249 is way smaller

    if last_updated_timestamp > current_timestamp - 60 {
        let _proposals = db.get_proposals().await;
        println!("Returning cached proposals");
        // ApiResponse should be Proposal struct Json(proposals)
        return Err(Status::NotImplemented);
    }

    println!("Fetching not yet indexed method calls from nearblocks");

    let nearblocks_client = nearblocks_client::ApiClient::default();

    let proposals = nearblocks_client
        .get_account_txns_by_pagination(
            "devhub.near".parse::<AccountId>().unwrap(),
            // Instead of just set_block_height_callback we should get all method calls
            // and handle them accordingly.
            Some("set_block_height_callback".to_string()),
            Some(timestamp_to_date_string(last_updated_timestamp)),
            // if this limit hits 10 we might need to do it in a loop let's say there are 100 changes since the last call to nearblocks.
            Some(10),
            Some("asc".to_string()),
        )
        .await;

    println!(
        "Fetched {} method calls from nearblocks",
        proposals.unwrap().len()
    );

    // TODO refactor this functionality away in nearblocks client
    let proposals_unwrapped = proposals.unwrap();
    let transaction = proposals_unwrapped
        .txns
        // don't get the first txn but all txns and than loop over them while inserting into postgres
        .first()
        .unwrap()
        .clone();
    let action = transaction.clone().actions.first().unwrap().clone();
    let json_args = action.args.clone();

    println!("json_args: {:?}", json_args.clone());
    let args: SetBlockHeightCallbackArgs = serde_json::from_str(&json_args).unwrap(); //.expect("Failed to parse json");

    println("Adding to the database...");
    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;
    DB::upsert_proposal(
        &mut tx,
        args.clone().proposal.id,
        args.clone().proposal.author_id.to_string(),
    )
    .await
    .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    println!("Added proposal to database, now adding timestamp.");

    let timestamp = args.proposal.snapshot.timestamp.try_into().unwrap();
    db.set_last_updated_timestamp(timestamp).await.unwrap();

    println!("Added timestamp to database, returning proposals...");

    // match args {
    //     Ok(proposal) => {
    //         println!("Fetched proposals from nearblocks");
    //         Ok(Json(proposal.proposal))
    //     }
    //     Err(e) => {
    //         println!("Failed to parse json: {:?}", e);
    //         Err(Status::InternalServerError)
    //     }
    // }

    // Upsert into postgres

    Ok(Json(args.proposal))
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

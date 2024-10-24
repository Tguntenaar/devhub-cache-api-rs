// use crate::db::DBTrait;
use devhub_cache_api::db::DB;
use devhub_cache_api::rpc_service::RpcService;
use devhub_cache_api::{nearblocks_client, timestamp_to_date_string};
use near::{types::Data, Contract, NetworkConfig};
use near_account_id::AccountId;
use rocket::request::FromParam;
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

// TODO add query params to get_proposals entrypoint
#[utoipa::path(get, path = "/proposals")]
#[get("/")]
async fn get_proposals(db: &State<DB>) -> Result<String, rocket::http::Status> {
    // Store in postgres
    // let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = db.begin().await.unwrap();
    // Upsert into postgres
    // proposals.clone().into_iter().for_each(|proposal| {
    //     let VersionedProposal::V0(proposal_v0) = proposal;
    // TODO: Upsert into postgres

    //     db.insert_proposal(&mut tx, "thomasguntenaar.near".to_string())
    //         .await
    //         .unwrap();
    // });
    // Get current timestamp
    let current_timestamp = chrono::Utc::now().timestamp();
    // Get last timestamp when database was updated
    let last_updated_timestamp = db.get_last_updated_timestamp().await.unwrap();

    // If last updated timestamp is within 1 minute return cached data from postgres
    if last_updated_timestamp > current_timestamp - 60 {
        let proposals = db.get_proposals().await;
        println!("Returning cached proposals");
        return Ok(format!("Hello, {:?}!", proposals));
    }
    println!("Fetching proposals from nearblocks");

    // Else fetch data from nearblocks and update database
    let nearblocks_client = nearblocks_client::ApiClient::default();
    // TODO should return proposals nog ApiResponse struct
    let proposals = nearblocks_client
        .get_account_txns_by_pagination(
            "devhub.near".parse::<AccountId>().unwrap(),
            Some("add_proposal".to_string()),
            Some(timestamp_to_date_string(last_updated_timestamp)),
            Some(10),
            Some("desc".to_string()),
        )
        .await;

    println!("Fetched proposals from nearblocks");
    // Upsert into postgres

    // TODO instead of rpc service use api client for nearblocks / database
    // let rpc_service = RpcService::new(Some("devhub.near".parse::<AccountId>().unwrap()));
    // let proposals = rpc_service.get_proposals().await;

    Ok(format!("Hello, {:?}!", proposals))
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

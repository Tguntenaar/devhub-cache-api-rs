use self::proposal_types::*;
use crate::db::db_types::ProposalWithLatestSnapshotView;
use crate::db::DB;
use crate::rpc_service::RpcService;
use crate::types::{Contract, PaginatedResponse};
use crate::{nearblocks_client, timestamp_to_date_string};
use devhub_shared::proposal::VersionedProposal;
use near_account_id::AccountId;
use rocket::serde::json::Json;
use rocket::{get, http::Status, State};
use std::convert::TryInto;
pub mod proposal_types;

// #[utoipa::path(get, path = "/proposals/search?<input>", params(
//   ("input"= &str, Path, description ="The string to search for in proposal name, description, summary, and category fields."),
// ))]
// #[get("/search/<input>")]
// fn search(input: String) -> Option<Json<PaginatedResponse<ProposalWithLatestSnapshotView>>> {
//     None
// }

#[utoipa::path(get, path = "/proposals?<order>&<limit>&<offset>&<filters>", params(
  ("order"= &str, Path, description ="order"),
  ("limit"= i64, Path, description = "limit"),
  ("offset"= i64, Path, description = "offset"),
  ("filters"= GetProposalFilters, Path, description = "filters struct that contains stuff like category, labels (vec), author_id, stage, block_timestamp (i64)"),
))]
#[get("/?<order>&<limit>&<offset>&<filters>")]
async fn get_proposals(
    order: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    filters: Option<GetProposalFilters>,
    db: &State<DB>,
    contract: &State<Contract>,
) -> Option<Json<PaginatedResponse<ProposalWithLatestSnapshotView>>> {
    let current_timestamp_nano = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let last_updated_timestamp = db.get_last_updated_timestamp().await.unwrap();

    let order = order.unwrap_or("desc");
    let limit = limit.unwrap_or(25);
    let offset = offset.unwrap_or(0);

    // If we called nearblocks in the last 60 milliseconds return the database values
    if current_timestamp_nano - last_updated_timestamp
        < chrono::Duration::seconds(60).num_nanoseconds().unwrap()
    {
        println!("Returning cached proposals");
        let (proposals, total) = match db
            .get_proposals_with_latest_snapshot(limit, order, offset, filters)
            .await
        {
            Err(e) => {
                println!("Failed to get proposals: {:?}", e);
                (vec![], 0)
            }
            Ok(result) => result,
        };

        return Some(Json(PaginatedResponse::new(
            proposals.into_iter().map(Into::into).collect(),
            1,
            limit.try_into().unwrap(),
            total.try_into().unwrap(),
        )));
    }

    println!("Fetching not yet indexed method calls from nearblocks");

    let nearblocks_client = nearblocks_client::ApiClient::default();

    // Nearblocks reacts with all contract changes since the timestamp we pass
    // This could return 0 new tx in which case we get the database stuff anyway
    // Or it could return 1 new tx in which case we want to update the database first
    // then get it from database using the right queries
    let nearblocks_unwrapped = match nearblocks_client
        .get_account_txns_by_pagination(
            contract.parse::<AccountId>().unwrap(),
            // Instead of just set_block_height_callback we should get all method calls
            // and handle them accordingly.
            // TODO change to None
            Some("set_block_height_callback".to_string()),
            Some(timestamp_to_date_string(last_updated_timestamp)),
            // if this limit hits 10 we might need to do it in a loop let's say there are 100 changes since the last call to nearblocks.
            Some(25),
            Some("asc".to_string()),
        )
        .await
    {
        Ok(nearblocks_unwrapped) => {
            // If the response was successful, print the count of method calls
            // println!("Response: {:?}", nearblocks_unwrapped);
            nearblocks_unwrapped
        }
        Err(e) => {
            // If there was an error, print it or handle it as needed
            eprintln!("Failed to fetch data from nearblocks: {:?}", e);
            nearblocks_client::ApiResponse { txns: vec![] }
        }
    };

    println!(
        "Fetched {} method calls from nearblocks",
        nearblocks_unwrapped.clone().txns.len()
    );

    let _ = nearblocks_client::transactions::process(&nearblocks_unwrapped.txns, db).await;

    match nearblocks_unwrapped
        .txns
        // should we get the first or last?
        .last()
    {
        Some(transaction) => {
            println!("Added proposals to database, now adding timestamp.");

            println!("Transaction timestamp: {}", transaction.block_timestamp);
            let timestamp_nano: i64 = transaction.block_timestamp.parse().unwrap();

            println!("Parsed tx timestamp: {}", timestamp_nano);
            db.set_last_updated_timestamp(timestamp_nano).await.unwrap();

            println!("Added timestamp to database, returning proposals...");
        }
        None => {
            println!("No transactions found")
        }
    };

    let (proposals, total) = match db
        .get_proposals_with_latest_snapshot(limit, order, offset, filters)
        .await
    {
        Err(e) => {
            println!("Failed to get proposals: {:?}", e);
            (vec![], 0)
        }
        Ok(result) => result,
    };

    Some(Json(PaginatedResponse::new(
        proposals.into_iter().map(Into::into).collect(),
        1,
        limit.try_into().unwrap(),
        total.try_into().unwrap(),
    )))
}

#[get("/test")]
async fn test(contract: &State<Contract>) -> String {
    format!("Welcome to {}", contract)
}

#[utoipa::path(get, path = "/proposals/{proposal_id}")]
#[get("/<proposal_id>")]
async fn get_proposal(proposal_id: i32) -> Result<Json<VersionedProposal>, rocket::http::Status> {
    let rpc_service = RpcService::default();
    // We should cache this in the future
    // We should also add rate limiting to this endpoint
    match rpc_service.get_proposal(proposal_id).await {
        Ok(proposal) => Ok(Json(proposal.data)),
        Err(e) => {
            eprintln!("Failed to get proposal from RPC: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

pub fn stage(contract: Contract) -> rocket::fairing::AdHoc {
    // rocket
    rocket::fairing::AdHoc::on_ignite("Proposal Stage", |rocket| async {
        println!("Proposal stage on ignite!");

        rocket.manage(contract).mount(
            "/proposals/",
            rocket::routes![get_proposals, get_proposal, test],
        )
    })
}

use self::proposal_types::*;
use crate::db::db_types::{ProposalSnapshotRecord, ProposalWithLatestSnapshotView};
use crate::db::DB;
use crate::nearblocks_client::types::Transaction;
use crate::rpc_service::RpcService;
use crate::types::PaginatedResponse;
use crate::{nearblocks_client, timestamp_to_date_string};
use devhub_shared::proposal::VersionedProposal;
use near_account_id::AccountId;
use rocket::serde::json::Json;
use rocket::{get, http::Status, State};
use std::convert::TryInto;
pub mod proposal_types;

// TODO input -> search name description summary fields
fn search() {}

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
) -> Option<Json<PaginatedResponse<ProposalWithLatestSnapshotView>>> {
    let current_timestamp_nano = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let last_updated_timestamp = db.get_last_updated_timestamp().await.unwrap();

    println!("last_updated_timestamp: {:?}", last_updated_timestamp);
    println!("current_timestamp: {:?}", current_timestamp_nano);

    println!(
        "Difference: {:?}",
        current_timestamp_nano - last_updated_timestamp
    );
    println!(
        "Duration: {:?}",
        chrono::Duration::seconds(60).num_nanoseconds().unwrap()
    );
    // If we called nearblocks in the last 60 milliseconds return the database values
    if current_timestamp_nano - last_updated_timestamp
        < chrono::Duration::seconds(60).num_nanoseconds().unwrap()
    {
        let _proposals = db.get_proposals().await;
        println!("Returning cached proposals");
        return None;
    }

    println!("Fetching not yet indexed method calls from nearblocks");

    let nearblocks_client = nearblocks_client::ApiClient::default();

    // Nearblocks reacts with all contract changes since the timestamp we pass
    // This could return 0 new tx in which case we get the database stuff anyway
    // Or it could return 1 new tx in which case we want to update the database first
    // then get it from database using the right queries
    let nearblocks_unwrapped = match nearblocks_client
        .get_account_txns_by_pagination(
            // TODO get from ENV variable
            "devhub.near".parse::<AccountId>().unwrap(),
            // Instead of just set_block_height_callback we should get all method calls
            // and handle them accordingly.
            // TODO no method call
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

    let _ = process_transactions(&nearblocks_unwrapped.txns, db).await;

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

    let order = order.unwrap_or("desc");
    let limit = limit.unwrap_or(25);
    let offset = offset.unwrap_or(0);

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

async fn process_transactions(transactions: &[Transaction], db: &State<DB>) -> Result<(), Status> {
    for transaction in transactions.iter() {
        if let Some(action) = transaction.actions.first() {
            let result = match action.method.as_str() {
                "set_block_height_callback" => {
                    handle_set_block_height_callback(transaction.to_owned(), db).await
                }
                "edit_proposal_versioned_timeline" => {
                    handle_edit_proposal(transaction.to_owned(), db).await
                }
                "edit_proposal_timeline" => handle_edit_proposal(transaction.to_owned(), db).await,
                "edit_proposal" => handle_edit_proposal(transaction.to_owned(), db).await,
                "edit_proposal_linked_rfp" => {
                    handle_edit_proposal(transaction.to_owned(), db).await
                }
                _ => {
                    println!("Unhandled method: {}", action.method);
                    continue;
                }
            };
            result?;
        }
    }

    Ok(())
}

async fn handle_set_block_height_callback(
    transaction: Transaction,
    db: &State<DB>,
) -> Result<(), Status> {
    let action = transaction.clone().actions.first().unwrap().clone();
    let json_args = action.args.clone();

    // println!("json_args: {:?}", json_args.clone());
    let args: SetBlockHeightCallbackArgs = serde_json::from_str(&json_args).unwrap();

    println!("Adding to the database... {}", args.clone().proposal.id);
    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;
    DB::upsert_proposal(
        &mut tx,
        args.clone().proposal.id,
        args.clone().proposal.author_id.to_string(),
    )
    .await
    .unwrap();

    let rpc_service = RpcService::default();
    let id = args.clone().proposal.id.try_into().unwrap();

    let versioned_proposal_fallback: VersionedProposal = args.clone().proposal.into();
    let versioned_proposal = match rpc_service.get_proposal(id).await {
        Ok(proposal) => proposal.data,
        Err(e) => {
            eprintln!(
                "Failed to get proposal from RPC, using first snapshot as fallback {:?}",
                e
            );
            versioned_proposal_fallback
        }
    };

    let snapshot = ProposalSnapshotRecord::from_contract_proposal(
        versioned_proposal.into(),
        transaction.block_timestamp,
        transaction.block.block_height,
    );

    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok(())
}

fn get_proposal_id(transaction: &Transaction) -> Result<i32, &'static str> {
    let action = transaction
        .actions
        .first()
        .ok_or("No actions found in transaction")?;

    let args: PartialEditProposalArgs = serde_json::from_str(&action.args).map_err(|e| {
        eprintln!("Failed to parse JSON: {:?}", e);
        "Failed to parse proposal arguments"
    })?;

    Ok(args.id)
}

async fn handle_edit_proposal(
    transaction: Transaction,
    db: &State<DB>,
) -> Result<(), rocket::http::Status> {
    let rpc_service = RpcService::default();
    let id = get_proposal_id(&transaction).map_err(|e| {
        eprintln!("Failed to get proposal ID: {}", e);
        Status::InternalServerError
    })?;
    let versioned_proposal = match rpc_service.get_proposal(id).await {
        Ok(proposal) => proposal.data,
        Err(e) => {
            eprintln!("Failed to get proposal from RPC: {:?}", e);
            return Err(Status::InternalServerError);
        }
    };

    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;

    let snapshot = ProposalSnapshotRecord::from_contract_proposal(
        versioned_proposal.into(),
        transaction.block_timestamp,
        transaction.block.block_height,
    );

    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok(())
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

pub fn stage() -> rocket::fairing::AdHoc {
    // rocket
    rocket::fairing::AdHoc::on_ignite("Proposal Stage", |rocket| async {
        println!("Proposal stage on ignite!");

        rocket.mount("/proposals/", rocket::routes![get_proposals, get_proposal])
    })
}

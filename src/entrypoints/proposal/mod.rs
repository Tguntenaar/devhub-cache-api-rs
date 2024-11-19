use self::proposal_types::*;
use crate::db::db_types::{ProposalSnapshotRecord, ProposalWithLatestSnapshotView};
use crate::db::DB;
use crate::rpc_service::RpcService;
use crate::types::PaginatedResponse;
use crate::{nearblocks_client, separate_number_and_text};
use devhub_shared::proposal::VersionedProposal;
use near_account_id::AccountId;
use rocket::serde::json::Json;
use rocket::{get, http::Status, State};
use std::convert::TryInto;
pub mod proposal_types;

// TODO use caching in search
#[utoipa::path(get, path = "/proposals/search?<input>", params(
  ("input"= &str, Path, description ="The string to search for in proposal name, description, summary, and category fields."),
))]
#[get("/search/<input>")]
async fn search(
    input: String,
    db: &State<DB>,
) -> Option<Json<PaginatedResponse<ProposalWithLatestSnapshotView>>> {
    let limit = 10;
    let (number, _) = separate_number_and_text(&input);

    let result = if let Some(number) = number {
        match db.get_proposal_with_latest_snapshot_by_id(number).await {
            Ok(proposal) => Ok((vec![proposal], 1)),
            Err(e) => Err(e),
        }
    } else {
        let search_input = format!("%{}%", input.to_lowercase());
        db.search_proposals_with_latest_snapshot(&search_input, limit, 0)
            .await
    };

    match result {
        Ok((proposals, total)) => Some(Json(PaginatedResponse::new(
            proposals.clone().into_iter().map(Into::into).collect(),
            1,
            limit.try_into().unwrap(),
            total.try_into().unwrap(),
        ))),
        Err(e) => {
            eprintln!("Error fetching proposals: {:?}", e);
            None
        }
    }
}

#[utoipa::path(get, path = "/proposals?<order>&<limit>&<offset>&<filters>", params(
  ("order"= &str, Path, description ="default order id_desc (ts_asc)"),
  ("limit"= i64, Path, description = "default limit 10"),
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
    contract: &State<AccountId>,
    nearblocks_api_key: &State<String>,
) -> Option<Json<PaginatedResponse<ProposalWithLatestSnapshotView>>> {
    let current_timestamp_nano = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let last_updated_info = db.get_last_updated_info().await.unwrap();

    let order = order.unwrap_or("id_desc");
    let limit = limit.unwrap_or(10);
    let offset = offset.unwrap_or(0);

    if current_timestamp_nano - last_updated_info.0
        < chrono::Duration::seconds(60).num_nanoseconds().unwrap()
    {
        println!("Returning cached proposals");
        let (proposals, total) = match db
            .get_proposals_with_latest_snapshot(limit, order, offset, filters)
            .await
        {
            Err(e) => {
                eprintln!("Failed to get proposals: {:?}", e);
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

    let nearblocks_client = nearblocks_client::ApiClient::new(nearblocks_api_key.inner().clone());

    let nearblocks_unwrapped = match nearblocks_client
        .get_account_txns_by_pagination(
            contract.inner().clone(),
            Some(last_updated_info.1),
            Some(50),
            Some("asc".to_string()),
            Some(1),
        )
        .await
    {
        Ok(nearblocks_unwrapped) => nearblocks_unwrapped,
        Err(e) => {
            eprintln!("Failed to fetch proposals from nearblocks: {:?}", e);
            nearblocks_client::ApiResponse { txns: vec![] }
        }
    };

    println!(
        "Fetched {} method calls from nearblocks",
        nearblocks_unwrapped.clone().txns.len()
    );

    let _ = nearblocks_client::transactions::process(
        &nearblocks_unwrapped.txns,
        db,
        contract.inner().clone(),
    )
    .await;

    match nearblocks_unwrapped.txns.last() {
        Some(transaction) => {
            let timestamp_nano: i64 = transaction.block_timestamp.parse().unwrap();

            db.set_last_updated_info(timestamp_nano, transaction.block.block_height)
                .await
                .unwrap();
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
            eprintln!("Failed to get proposals: {:?}", e);
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

#[utoipa::path(get, path = "/proposal/{proposal_id}/snapshots")]
#[get("/<proposal_id>/snapshots")]
async fn get_proposal_with_all_snapshots(
    proposal_id: i32,
    db: &State<DB>,
) -> Result<Json<Vec<ProposalSnapshotRecord>>, Status> {
    match db.get_proposal_with_all_snapshots(proposal_id).await {
        Err(e) => {
            eprintln!("Failed to get proposal snapshots: {:?}", e);
            // Ok(Json(vec![]))
            Err(Status::InternalServerError)
        }
        Ok(result) => Ok(Json(result)),
    }
}

#[get("/info/<block_height>")]
async fn set_timestamp(block_height: i64, db: &State<DB>) -> Result<(), Status> {
    match db.set_last_updated_info(0, block_height).await {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error updating timestamp: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/info")]
async fn get_timestamp(db: &State<DB>) -> Result<Json<(i64, i64)>, Status> {
    let (timestamp, block_height) = db.get_last_updated_info().await.unwrap();
    Ok(Json((timestamp, block_height)))
}

#[utoipa::path(get, path = "/proposal/{proposal_id}")]
#[get("/<proposal_id>")]
async fn get_proposal(
    proposal_id: i32,
    contract: &State<AccountId>,
) -> Result<Json<VersionedProposal>, rocket::http::Status> {
    let rpc_service = RpcService::new(contract.inner().clone());
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

        rocket
            .mount(
                "/proposals/",
                rocket::routes![get_proposals, set_timestamp, get_timestamp, search],
            )
            .mount(
                "/proposal/",
                rocket::routes![get_proposal, get_proposal_with_all_snapshots],
            )
    })
}

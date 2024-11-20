use self::rfp_types::*;
use crate::db::db_types::{RfpSnapshotRecord, RfpWithLatestSnapshotView};
use crate::db::DB;
use crate::rpc_service::RpcService;
use crate::types::PaginatedResponse;
use crate::{nearblocks_client, separate_number_and_text, timestamp_to_date_string};
use devhub_shared::rfp::VersionedRFP;
use near_account_id::AccountId;
use rocket::serde::json::Json;
use rocket::{delete, get, http::Status, State};
use std::convert::TryInto;
pub mod rfp_types;

// TODO use caching in search
#[utoipa::path(get, path = "/rfps/search?<input>", params(
  ("input"= &str, Path, description ="The string to search for in rfp name, description, summary, and category fields."),
))]
#[get("/search/<input>")]
async fn search(
    input: &str,
    db: &State<DB>,
) -> Option<Json<PaginatedResponse<RfpWithLatestSnapshotView>>> {
    let limit = 10;
    let (number_opt, _) = separate_number_and_text(input);
    let result = if let Some(number) = number_opt {
        match db.get_rfp_with_latest_snapshot_by_id(number).await {
            Ok(rfp) => Ok((vec![rfp], 1)),
            Err(e) => Err(e),
        }
    } else {
        let search_input = format!("%{}%", input.to_lowercase());
        db.search_rfps_with_latest_snapshot(&search_input, limit, 0)
            .await
    };

    match result {
        Ok((rfps, total)) => Some(Json(PaginatedResponse::new(
            rfps.into_iter().map(Into::into).collect(),
            1,
            limit.try_into().unwrap(),
            total.try_into().unwrap(),
        ))),
        Err(e) => {
            eprintln!("Error fetching rfps: {:?}", e);
            None
        }
    }
}

#[utoipa::path(get, path = "/rfps?<order>&<limit>&<offset>&<filters>", params(
  ("order"= &str, Path, description ="default order id_desc"),
  ("limit"= i64, Path, description = "default limit 10"),
  ("offset"= i64, Path, description = "offset"),
  ("filters"= GetRfpFilters, Path, description = "filters struct that contains stuff like category, labels (vec), author_id, stage, block_timestamp (i64)"),
))]
#[get("/?<order>&<limit>&<offset>&<filters>")]
async fn get_rfps(
    order: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    filters: Option<GetRfpFilters>,
    db: &State<DB>,
    contract: &State<AccountId>,
    nearblocks_api_key: &State<String>,
) -> Option<Json<PaginatedResponse<RfpWithLatestSnapshotView>>> {
    let current_timestamp_nano = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let last_updated_info = db.get_last_updated_info().await.unwrap();

    // A day in nanos
    println!(
        "Last updated timestamp date: {}",
        timestamp_to_date_string(last_updated_info.0)
    );

    let order = order.unwrap_or("id_desc");
    let limit = limit.unwrap_or(10);
    let offset = offset.unwrap_or(0);

    if current_timestamp_nano - last_updated_info.0
        < chrono::Duration::seconds(60).num_nanoseconds().unwrap()
    {
        println!("Returning cached proposals");
        let (rfps, total) = match db
            .get_rfps_with_latest_snapshot(limit, order, offset, filters)
            .await
        {
            Err(e) => {
                eprintln!("Failed to get proposals: {:?}", e);
                (vec![], 0)
            }
            Ok(result) => result,
        };

        return Some(Json(PaginatedResponse::new(
            rfps.into_iter().map(Into::into).collect(),
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
            eprintln!("Failed to fetch rfp from nearblocks: {:?}", e);
            nearblocks_client::ApiResponse { txns: vec![] }
        }
    };

    let _ = nearblocks_client::transactions::process(
        &nearblocks_unwrapped.txns,
        db,
        contract.inner().clone(),
    )
    .await;

    // TODO: Check if the last transaction is the same day as the last updated timestamp
    // If it is, then we need to use the cursor from the nearblocks response
    match nearblocks_unwrapped.txns.last() {
        Some(transaction) => {
            let timestamp_nano: i64 = transaction.receipt_block.block_timestamp;
            db.set_last_updated_info(timestamp_nano, transaction.block.block_height)
                .await
                .unwrap();
        }
        None => {
            println!("No transactions found")
        }
    };

    let (rfps, total) = match db
        .get_rfps_with_latest_snapshot(limit, order, offset, filters)
        .await
    {
        Err(e) => {
            eprintln!("Failed to get rfps: {:?}", e);
            (vec![], 0)
        }
        Ok(result) => result,
    };

    Some(Json(PaginatedResponse::new(
        rfps.into_iter().map(Into::into).collect(),
        1,
        limit.try_into().unwrap(),
        total.try_into().unwrap(),
    )))
}

#[utoipa::path(get, path = "/rfp/{rfp_id}")]
#[get("/<rfp_id>")]
async fn get_rfp(rfp_id: i32, contract: &State<AccountId>) -> Result<Json<VersionedRFP>, Status> {
    // TODO Get cached rfp
    match RpcService::new(contract.inner().clone())
        .get_rfp(rfp_id)
        .await
    {
        Ok(rfp) => Ok(Json(rfp.data)),
        Err(e) => {
            eprintln!("In /rfp/rfp_id; Failed to get rfp from RPC: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[utoipa::path(get, path = "/{rfp_id}/snapshots")]
#[get("/<rfp_id>/snapshots")]
async fn get_rfp_with_snapshots(
    rfp_id: i64,
    db: &State<DB>,
) -> Result<Json<Vec<RfpSnapshotRecord>>, Status> {
    match db.get_rfp_with_all_snapshots(rfp_id).await {
        Err(e) => {
            eprintln!("Failed to get rfps: {:?}", e);
            // Ok(Json(vec![]))
            Err(Status::InternalServerError)
        }
        Ok((result, _)) => Ok(Json(result)),
    }
}

// TODO Remove this once we go in production or put it behind authentication or a flag
#[delete("/<rfp_id>/snapshots")]
async fn remove_rfp_snapshots_by_rfp_id(rfp_id: i32, db: &State<DB>) -> Result<(), Status> {
    match db.remove_rfp_snapshots_by_rfp_id(rfp_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to remove rfp snapshots: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("Rfp Stage", |rocket| async {
        println!("Rfp stage on ignite!");

        rocket
            .mount("/rfps/", rocket::routes![get_rfps, search])
            .mount(
                "/rfp/",
                rocket::routes![
                    get_rfp,
                    get_rfp_with_snapshots,
                    remove_rfp_snapshots_by_rfp_id
                ],
            )
    })
}

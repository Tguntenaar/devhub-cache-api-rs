use devhub_cache_api::nearblocks_client::types::Transaction;
use devhub_cache_api::{db::DB, rpc_service::RpcService};
use devhub_shared::rfp::VersionedRFP;
use rocket::{get, http::Status, serde::json::Json, FromForm, State};
use serde::Deserialize;

// Struct for query parameters
#[derive(Debug, FromForm)]
struct RfpQuery {
    limit: Option<usize>, // Optional limit parameter
    sort: Option<String>, // Optional sorting parameter
}

#[derive(Deserialize)]
pub struct PartialEditRFPArgs {
    pub id: i32,
}

fn get_rfp_id(transaction: &Transaction) -> Result<i32, &'static str> {
    let action = transaction
        .actions
        .first()
        .ok_or("No actions found in transaction")?;

    let args: PartialEditRFPArgs = serde_json::from_str(&action.args).map_err(|e| {
        eprintln!("Failed to parse JSON: {:?}", e);
        "Failed to parse proposal arguments"
    })?;

    Ok(args.id)
}

#[utoipa::path(get, path = "/rfps")]
#[get("/")]
async fn get_rfps(db: &State<DB>) -> Result<String, Status> {
    // methods to handle:
    // "edit_rfp", "edit_rfp_timeline", "edit_rfp_internal", "cancel_rfp"
    // callbacks: to be handled in rfp
    // "set_rfp_block_height_callback"
    Ok(format!("Hello, {:?}!", "rfps"))
}

async fn handle_edit_rfp(transaction: Transaction, db: &State<DB>) -> Result<(), Status> {
    let rpc_service = RpcService::default();
    let id = get_rfp_id(&transaction).map_err(|e| {
        eprintln!("Failed to get RFP ID: {}", e);
        Status::InternalServerError
    })?;

    let versioned_rfp = match rpc_service.get_rfp(id).await {
        Ok(rfp) => Ok(Json(rfp)),
        Err(e) => {
            eprintln!("Failed to get rfp from RPC: {:?}", e);
            Err(Status::InternalServerError)
        }
    };

    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;

    // TODO
    // DB::insert_rfp_snapshot(&mut tx, &versioned_rfp.into())
    //     .await
    //     .map_err(|_e| Status::InternalServerError)?;

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok(())
}
#[utoipa::path(get, path = "/rfps/{rfp_id}")]
#[get("/<rfp_id>")]
async fn get_rfp(rfp_id: i32) -> Result<Json<VersionedRFP>, Status> {
    match RpcService::default().get_rfp(rfp_id).await {
        Ok(rfp) => Ok(Json(rfp)),
        Err(e) => {
            eprintln!("Failed to get rfp from RPC: {:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("Rfp Stage", |rocket| async {
        println!("Rfp stage on ignite!");

        rocket.mount("/rfps/", rocket::routes![get_rfps, get_rfp])
    })
}

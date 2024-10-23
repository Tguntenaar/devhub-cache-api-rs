use devhub_cache_api::db::DB;
use devhub_shared::rfp::VersionedRFP;
use rocket::{get, FromForm, State};

// Struct for query parameters
#[derive(Debug, FromForm)]
struct RfpQuery {
    limit: Option<usize>, // Optional limit parameter
    sort: Option<String>, // Optional sorting parameter
}

#[utoipa::path(get, path = "/rfps")]
#[get("/")]
async fn get_rfps(db: &State<DB>) -> Result<String, rocket::http::Status> {
    Ok(format!("Hello, {:?}!", "rfps"))
}

#[utoipa::path(get, path = "/rfps/{rfp_id}")]
#[get("/<rfp_id>")]
async fn get_rfp(rfp_id: i32, db: &State<DB>) -> Result<String, rocket::http::Status> {
    Ok(format!("Hello, {:?}!", rfp_id))
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("Rfp Stage", |rocket| async {
        println!("Rfp stage on ignite!");

        rocket.mount("/rfps/", rocket::routes![get_rfps, get_rfp])
    })
}

use std::sync::Arc;

use near::{types::Data, Contract, NetworkConfig};
use near_account_id::AccountId;
use rocket::http::uri::Query;
use rocket::request::FromParam;

use rocket::serde::Serialize;
use rocket::{catch, catchers, get, launch, routes, FromForm};
use serde_json::json;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod entrypoints;
use entrypoints::ApiDoc;

use devhub_cache_api::db;
use rocket_cors::AllowedOrigins;
use rocket_db_pools::sqlx::{self, PgPool};

use rocket::serde::json::Json;
use rocket_db_pools::{Connection, Database};

#[get("/")]
fn index() -> &'static str {
    "Welcome from fly.io!!!!!"
}

// Allow robots to crawl the site
#[get("/robots.txt")]
fn robots() -> &'static str {
    "User-agent: *\nDisallow: /"
}

#[catch(422)]
fn unprocessable_entity() -> &'static str {
    "Custom 422 Error: Unprocessable Entity"
}

#[catch(500)]
fn internal_server_error() -> &'static str {
    "Custom 500 Error: Internal Server Error"
}

#[catch(404)]
fn not_found() -> &'static str {
    "Custom 404 Error: Not Found"
}

#[catch(400)]
fn bad_request() -> &'static str {
    "Custom 400 Error: Bad Request"
}

#[derive(Debug, serde::Deserialize)]
pub struct Env {
    contract: String,
    database_url: String,
}

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let atomic_bool = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // let env = envy::from_env::<Env>().expect("Failed to load environment variables");
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // PgConnection::establish(&env.database_url)
    //     .unwrap_or_else(|_| panic!("Error connecting to {}", env.database_url));

    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        // TODO Add prod urls here
    ]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create cors config");

    rocket::build()
        .attach(cors)
        .attach(db::stage())
        .mount("/", routes![robots, index])
        // .mount("/", routes![get_all_proposal_ids, get_proposals])
        .attach(entrypoints::stage())
        // TODO add fairing/ middleware background service that is callable from entrypoints
        // also as a cron job
        .attach(rocket::fairing::AdHoc::on_shutdown(
            "Stop loading users from Near and Github metadata",
            |_| {
                Box::pin(async move {
                    atomic_bool.store(false, std::sync::atomic::Ordering::Relaxed);
                })
            },
        ))
        .mount(
            "/",
            SwaggerUi::new("/swagger-ui/<_..>").url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .register(
            "/",
            catchers![
                unprocessable_entity,
                internal_server_error,
                not_found,
                bad_request
            ],
        )
}

pub mod api_background_service;
pub mod db;
pub mod entrypoints;
pub mod nearblocks_client;
pub mod rpc_service;
pub mod types;

use chrono::DateTime;

pub fn timestamp_to_date_string(timestamp: i64) -> String {
    // Convert the timestamp to a NaiveDateTime
    let datetime = DateTime::from_timestamp_nanos(timestamp);

    // Format the NaiveDateTime to a string in YYYY-MM-DD format
    datetime.format("%Y-%m-%d").to_string()
}

use crate::entrypoints::ApiDoc;
use rocket::{catch, catchers, get, launch, routes};
use rocket_cors::AllowedOrigins;
use types::Contract;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[cfg(test)]
mod tests;

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
    contract: Contract,
    database_url: String,
}

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();

    let env: Env = envy::from_env::<Env>().expect("Failed to load environment variables");

    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "http://127.0.0.1:8080",
        "https://dev.near.org",
        "https://near.social",
        "https://neardevhub.org",
        "https://devhub.near.page",
        "https://devhub-cache-api-rs.fly.dev",
        "https://infra-cache-api-rs.fly.dev",
        "https://events-cache-api-rs.fly.dev",
        // TODO Add prod urls here
    ]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create cors config");

    let figment = rocket::Config::figment()
        .merge(rocket::Config::default())
        .merge(("databases.my_db.url", env.database_url));

    rocket::custom(figment)
        .attach(cors)
        .attach(db::stage())
        .mount("/", routes![robots, index])
        .attach(entrypoints::stage(env.contract))
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

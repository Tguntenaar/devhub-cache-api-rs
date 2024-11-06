use rocket::{catch, catchers, get, launch, routes};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod entrypoints;
use devhub_cache_api::db;
use entrypoints::ApiDoc;
use rocket_cors::AllowedOrigins;

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

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let atomic_bool = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "http://127.0.0.1:8080",
        "https://dev.near.org",
        "https://near.social",
        "https://neardevhub.org",
        "https://devhub.near.page",
        "https://devhub-cache-api-rs.fly.dev", // TODO Add prod urls here
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
        .attach(entrypoints::stage())
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

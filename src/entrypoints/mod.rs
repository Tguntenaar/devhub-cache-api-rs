use rocket::fairing::AdHoc;
use utoipa::OpenApi;
pub mod proposal;
// pub mod rfp;
use crate::db::types::ProposalWithLatestSnapshotView;

use devhub_cache_api::types;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Devhub Cache API",
        version = "0.0.1",
    ),
    paths(
      proposal::get_proposals
    ),
    components(schemas(
      types::PaginatedResponse<ProposalWithLatestSnapshotView>
    )),
    tags(
        (name = "Devhub Cache", description = "Devhub cache endpoints.")
    ),
)]
pub struct ApiDoc;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Installing entrypoints", |rocket| async {
        rocket.attach(proposal::stage())
        // .attach(rfp::stage())
    })
}

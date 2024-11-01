use rocket::fairing::AdHoc;
use utoipa::OpenApi;

// pub mod aliases;
// pub mod leaderboards;
// pub mod statistics;
// pub mod user;
pub mod proposal;
pub mod rfp;
use crate::db::types::{ProposalRecord, ProposalSnapshotRecord};

use devhub_cache_api::types;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Devhub Cache API",
        version = "0.0.1",
    ),
    paths(
        // leaderboards::get_leaderboard,
        // leaderboards::get_repos,
        // user::get_user,
        // user::get_user_contributions,
        // user::get_badge,
        // statistics::get_statistics
        proposal::get_proposals
    ),
    components(schemas(
        // types::PaginatedResponse<types::LeaderboardResponse>,
        // types::PaginatedLeaderboardResponse,
        // types::PaginatedResponse<types::RepoResponse>,
        // types::PaginatedRepoResponse,
        // types::PaginatedResponse<types::UserContributionResponse>,
        // types::PaginatedUserContributionResponse,
        // types::UserContributionResponse,
        // types::LeaderboardResponse,
        // types::RepoResponse,
        // types::UserProfile,
        // types::GithubMeta,
        // types::Streak,
        // types::Statistics,
        types::PaginatedResponse<ProposalRecord>
    )),
    tags(
        (name = "Devhub Cache", description = "Devhub cache endpoints.")
    ),
)]
pub struct ApiDoc;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Installing entrypoints", |rocket| async {
        rocket.attach(proposal::stage()).attach(rfp::stage())
        // .attach(aliases::stage())
        // .attach(statistics::stage())
    })
}

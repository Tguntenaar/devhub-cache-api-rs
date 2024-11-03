use devhub_cache_api::db::DB;
use devhub_cache_api::{nearblocks_client, timestamp_to_date_string};
use devhub_shared::proposal::{Proposal, ProposalFundingCurrency, VersionedProposalBody};
use near_account_id::AccountId;
use rocket::request::FromParam;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, http::Status, FromForm, State};
use std::collections::HashSet;

use std::convert::TryInto;

// Assuming these are the types you are working with
use devhub_cache_api::db::types::ProposalSnapshotRecord;
use devhub_shared::proposal::Proposal as ContractProposal;

// Define a trait for accessing various fields
trait ProposalBodyFields {
    fn get_name(&self) -> &String;
    fn get_category(&self) -> &String;
    fn get_summary(&self) -> &String;
    fn get_description(&self) -> &String;
    fn get_linked_proposals(&self) -> &Vec<u32>;
    fn get_requested_sponsorship_usd_amount(&self) -> &u32;
    fn get_requested_sponsorship_paid_in_currency(&self) -> String;
    fn get_requested_sponsor(&self) -> String;
    fn get_receiver_account(&self) -> String;
    fn get_supervisor(&self) -> Option<String>;
    fn get_timeline(&self) -> String;
    fn get_linked_rfp(&self) -> &Option<u32>;
}

trait ProposalFundingCurrencyToString {
    fn to_string(&self) -> String;
}

// Implement the new trait for ProposalFundingCurrency
impl ProposalFundingCurrencyToString for ProposalFundingCurrency {
    fn to_string(&self) -> String {
        match self {
            ProposalFundingCurrency::NEAR => "NEAR".to_string(),
            ProposalFundingCurrency::USDT => "USDT".to_string(),
            ProposalFundingCurrency::USDC => "USDC".to_string(),
            ProposalFundingCurrency::OTHER => "OTHER".to_string(),
        }
    }
}
// Implement the trait for VersionedProposalBody
impl ProposalBodyFields for VersionedProposalBody {
    fn get_name(&self) -> &String {
        match self {
            VersionedProposalBody::V0(body) => &body.name,
            VersionedProposalBody::V1(body) => &body.name,
            VersionedProposalBody::V2(body) => &body.name,
        }
    }

    fn get_category(&self) -> &String {
        match self {
            VersionedProposalBody::V0(body) => &body.category,
            VersionedProposalBody::V1(body) => &body.category,
            VersionedProposalBody::V2(body) => &body.category,
        }
    }

    fn get_summary(&self) -> &String {
        match self {
            VersionedProposalBody::V0(body) => &body.summary,
            VersionedProposalBody::V1(body) => &body.summary,
            VersionedProposalBody::V2(body) => &body.summary,
        }
    }

    fn get_description(&self) -> &String {
        match self {
            VersionedProposalBody::V0(body) => &body.description,
            VersionedProposalBody::V1(body) => &body.description,
            VersionedProposalBody::V2(body) => &body.description,
        }
    }

    fn get_linked_proposals(&self) -> &Vec<u32> {
        match self {
            VersionedProposalBody::V0(body) => &body.linked_proposals,
            VersionedProposalBody::V1(body) => &body.linked_proposals,
            VersionedProposalBody::V2(body) => &body.linked_proposals,
        }
    }

    fn get_linked_rfp(&self) -> &Option<u32> {
        match self {
            VersionedProposalBody::V0(_) => &None,
            VersionedProposalBody::V1(_) => &None,
            VersionedProposalBody::V2(body) => &body.linked_rfp,
        }
    }

    fn get_requested_sponsorship_usd_amount(&self) -> &u32 {
        match self {
            VersionedProposalBody::V0(body) => &body.requested_sponsorship_usd_amount,
            VersionedProposalBody::V1(body) => &body.requested_sponsorship_usd_amount,
            VersionedProposalBody::V2(body) => &body.requested_sponsorship_usd_amount,
        }
    }

    fn get_requested_sponsorship_paid_in_currency(&self) -> String {
        match self {
            VersionedProposalBody::V0(body) => {
                body.requested_sponsorship_paid_in_currency.to_string()
            }
            VersionedProposalBody::V1(body) => {
                body.requested_sponsorship_paid_in_currency.to_string()
            }
            VersionedProposalBody::V2(body) => {
                body.requested_sponsorship_paid_in_currency.to_string()
            }
        }
    }

    fn get_requested_sponsor(&self) -> String {
        match self {
            VersionedProposalBody::V0(body) => body.requested_sponsor.to_string(),
            VersionedProposalBody::V1(body) => body.requested_sponsor.to_string(),
            VersionedProposalBody::V2(body) => body.requested_sponsor.to_string(),
        }
    }

    fn get_receiver_account(&self) -> String {
        match self {
            VersionedProposalBody::V0(body) => body.receiver_account.to_string(),
            VersionedProposalBody::V1(body) => body.receiver_account.to_string(),
            VersionedProposalBody::V2(body) => body.receiver_account.to_string(),
        }
    }

    fn get_supervisor(&self) -> Option<String> {
        match self {
            VersionedProposalBody::V0(body) => body.supervisor.as_ref().map(|id| id.to_string()),
            VersionedProposalBody::V1(body) => body.supervisor.as_ref().map(|id| id.to_string()),
            VersionedProposalBody::V2(body) => body.supervisor.as_ref().map(|id| id.to_string()),
        }
    }

    fn get_timeline(&self) -> String {
        match self {
            VersionedProposalBody::V0(body) => {
                serde_json::to_string(&body.timeline).unwrap_or_default()
            }
            VersionedProposalBody::V1(body) => {
                serde_json::to_string(&body.timeline).unwrap_or_default()
            }
            VersionedProposalBody::V2(body) => {
                serde_json::to_string(&body.timeline).unwrap_or_default()
            }
        }
    }
    // Implement more methods as needed
}

// Define a trait for the conversion
trait FromContractProposal {
    fn from_contract_proposal(
        proposal: ContractProposal,
        timestamp: String,
        block_height: i64,
    ) -> Self;
}

impl FromContractProposal for ProposalSnapshotRecord {
    fn from_contract_proposal(
        proposal: ContractProposal,
        timestamp: String,
        block_height: i64,
    ) -> Self {
        ProposalSnapshotRecord {
            proposal_id: proposal.id as i32,
            block_height,
            ts: timestamp.parse::<i32>().unwrap_or_default(),
            editor_id: proposal.snapshot.editor_id.to_string(),
            social_db_post_block_height: proposal.social_db_post_block_height as i64,
            labels: serde_json::Value::from(Vec::from_iter(
                proposal.snapshot.labels.iter().cloned(),
            )),
            proposal_version: "V0".to_string(),
            proposal_body_version: "V2".to_string(),
            name: Some(proposal.snapshot.body.get_name().clone()),
            category: Some(proposal.snapshot.body.get_category().clone()),
            summary: Some(proposal.snapshot.body.get_summary().clone()),
            description: Some(proposal.snapshot.body.get_description().clone()),
            linked_proposals: Some(serde_json::Value::from(Vec::from_iter(
                proposal.snapshot.body.get_linked_proposals().to_vec(),
            ))),
            linked_rfp: proposal.snapshot.body.get_linked_rfp().map(|x| x as i32),
            requested_sponsorship_usd_amount: Some(
                *proposal
                    .snapshot
                    .body
                    .get_requested_sponsorship_usd_amount() as i32,
            ),
            requested_sponsorship_paid_in_currency: Some(
                proposal
                    .snapshot
                    .body
                    .get_requested_sponsorship_paid_in_currency()
                    .clone(),
            ),
            requested_sponsor: Some(proposal.snapshot.body.get_requested_sponsor().clone()),
            receiver_account: Some(proposal.snapshot.body.get_receiver_account().clone()),
            supervisor: proposal.snapshot.body.get_supervisor(),
            timeline: Some(serde_json::Value::from(
                proposal.snapshot.body.get_timeline().clone(),
            )),
            views: None,
        }
    }
}

#[derive(Debug, Serialize)]
struct ProposalIds(Vec<i32>);

impl<'r> FromParam<'r> for ProposalIds {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        let ids = param
            .split(',')
            .map(|s| s.parse::<i32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Invalid integer")?;
        Ok(ProposalIds(ids))
    }
}

// Struct for query parameters
#[derive(Debug, FromForm)]
pub struct ProposalQuery {
    proposal_ids: Option<Vec<i32>>,
    limit: Option<usize>, // Optional limit parameter
    sort: Option<String>, // Optional sorting parameter
}

#[derive(Serialize, Deserialize)]
struct AddProposalArgs {
    body: VersionedProposalBody,
    labels: HashSet<String>,
    accepted_terms_and_conditions_version: Option<near_sdk::BlockHeight>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SetBlockHeightCallbackArgs {
    proposal: Proposal,
}

// add query params to get_proposals entrypoint
#[utoipa::path(get, path = "/proposals")]
#[get("/")]
async fn get_proposals(db: &State<DB>) -> Result<Json<Proposal>, Status> {
    // Get current timestamp
    // let current_timestamp = chrono::Utc::now().timestamp();
    let current_timestamp = chrono::Utc::now().timestamp_millis();
    // Get last timestamp when database was updated
    let last_updated_timestamp = db.get_last_updated_timestamp().await.unwrap();

    // TODO the timestamps are way off between blockchain and database.
    println!("last_updated_timestamp: {:?}", last_updated_timestamp); // 1709470463748732291
    println!("current_timestamp: {:?}", current_timestamp); // 1729806249 is way smaller

    // If we called nearblocks in the last 60 seconds return the database values
    if last_updated_timestamp > current_timestamp - 60 {
        let _proposals = db.get_proposals().await;
        println!("Returning cached proposals");
        // ApiResponse should be Proposal struct Json(proposals)
        return Err(Status::NotImplemented);
    }

    println!("Fetching not yet indexed method calls from nearblocks");

    let nearblocks_client = nearblocks_client::ApiClient::default();

    // Nearblocks reacts with all contract changes since the timestamp we pass
    // This could return 0 new tx in which case we get the database stuff anyway
    // Or it could return 1 new tx in which case we want to update the database first
    // then get it from database using the right queries
    let nearblocks_response = nearblocks_client
        .get_account_txns_by_pagination(
            "devhub.near".parse::<AccountId>().unwrap(),
            // Instead of just set_block_height_callback we should get all method calls
            // and handle them accordingly.
            Some("set_block_height_callback".to_string()),
            Some(timestamp_to_date_string(last_updated_timestamp)),
            // if this limit hits 10 we might need to do it in a loop let's say there are 100 changes since the last call to nearblocks.
            Some(10),
            Some("asc".to_string()),
        )
        .await;

    // TODO handle these different method calls
    // "edit_proposal",
    // "edit_proposal_internal",
    // "edit_proposal_linked_rfp",
    // "edit_proposal_timeline",
    // "edit_proposal_versioned_timeline",

    let nearblocks_unwrapped = nearblocks_response.unwrap();

    println!(
        "Fetched {} method calls from nearblocks",
        nearblocks_unwrapped.clone().txns.len()
    );

    // TODO refactor this functionality away in nearblocks client
    let transaction = nearblocks_unwrapped
        .txns
        // don't get the first txn but all txns and than loop over them while inserting into postgres
        .first()
        .unwrap()
        .clone();
    let action = transaction.clone().actions.first().unwrap().clone();
    let json_args = action.args.clone();

    println!("json_args: {:?}", json_args.clone());
    let args: SetBlockHeightCallbackArgs = serde_json::from_str(&json_args).unwrap();

    println!("Adding to the database...");
    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;
    DB::upsert_proposal(
        &mut tx,
        args.clone().proposal.id,
        args.clone().proposal.author_id.to_string(),
    )
    .await
    .unwrap();

    let block_timestamp = transaction.clone().block_timestamp;
    let block_height = transaction.clone().block.block_height;

    let snapshot: devhub_cache_api::db::types::ProposalSnapshotRecord =
        FromContractProposal::from_contract_proposal(
            args.proposal.clone(),
            block_timestamp,
            block_height,
        );

    //
    DB::upsert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    println!("Added proposal to database, now adding timestamp.");

    let timestamp = args.proposal.snapshot.timestamp.try_into().unwrap();
    db.set_last_updated_timestamp(timestamp).await.unwrap();

    println!("Added timestamp to database, returning proposals...");

    // match args {
    //     Ok(proposal) => {
    //         println!("Fetched proposals from nearblocks");
    //         Ok(Json(proposal.proposal))
    //     }
    //     Err(e) => {
    //         println!("Failed to parse json: {:?}", e);
    //         Err(Status::InternalServerError)
    //     }
    // }
    // Upsert into postgres

    Ok(Json(args.proposal))
}

async fn handle_set_block_height() {}

async fn handle_edit_proposal() {}

async fn handle_edit_proposal_timeline() {}

#[utoipa::path(get, path = "/proposals/{proposal_id}")]
#[get("/<proposal_id>")]
async fn get_proposal(proposal_id: i32, db: &State<DB>) -> Result<String, rocket::http::Status> {
    Ok(format!("Hello, {:?}!", proposal_id))
}

pub fn stage() -> rocket::fairing::AdHoc {
    // rocket
    rocket::fairing::AdHoc::on_ignite("Proposal Stage", |rocket| async {
        println!("Proposal stage on ignite!");

        rocket.mount("/proposals/", rocket::routes![get_proposals, get_proposal])
    })
}

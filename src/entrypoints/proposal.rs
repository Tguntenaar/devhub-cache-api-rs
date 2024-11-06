use devhub_cache_api::api_client::ApiResponse;
use devhub_cache_api::db::DB;
use devhub_cache_api::nearblocks_client::types::Transaction;
use devhub_cache_api::rpc_service::RpcService;
use devhub_cache_api::types::PaginatedResponse;
use devhub_cache_api::{nearblocks_client, timestamp_to_date_string};
use devhub_shared::proposal::{
    Proposal, ProposalFundingCurrency, ProposalId, VersionedProposal, VersionedProposalBody,
};
use near_account_id::AccountId;
use near_sdk::near;
use rocket::request::FromParam;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, http::Status, FromForm, State};
use std::collections::HashSet;

use std::convert::TryInto;

// Assuming these are the types you are working with
use devhub_cache_api::db::types::{ProposalSnapshotRecord, ProposalWithLatestSnapshotView};
// TODO think about if this should be VersionedProposal instead of Proposal :(
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
            ts: timestamp.parse::<i64>().unwrap_or_default(),
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

trait FromVersionedProposalBody {
    fn from_versioned_proposal_body(
        args: EditProposalArgs,
        timestamp: String,
        block_height: i64,
        editor_id: String,
    ) -> Self;
}

impl FromVersionedProposalBody for ProposalSnapshotRecord {
    fn from_versioned_proposal_body(
        args: EditProposalArgs,
        timestamp: String,
        block_height: i64,
        editor_id: String, // transaction.predecessor_account_id
    ) -> Self {
        ProposalSnapshotRecord {
            proposal_id: args.id as i32,
            block_height,
            ts: timestamp.parse::<i64>().unwrap_or_default(),
            editor_id,
            social_db_post_block_height: 0,
            labels: serde_json::Value::from(Vec::from_iter(args.labels.iter().cloned())),
            proposal_version: "V0".to_string(), // Get this from the last snapshot
            proposal_body_version: "V2".to_string(), // Get this from the last snapshot
            name: Some(args.body.get_name().clone()),
            category: Some(args.body.get_category().clone()),
            summary: Some(args.body.get_summary().clone()),
            description: Some(args.body.get_description().clone()),
            linked_proposals: Some(serde_json::Value::from(Vec::from_iter(
                args.body.get_linked_proposals().to_vec(),
            ))),
            linked_rfp: args.body.get_linked_rfp().map(|x| x as i32),
            requested_sponsorship_usd_amount: Some(
                *args.body.get_requested_sponsorship_usd_amount() as i32,
            ),
            requested_sponsorship_paid_in_currency: Some(
                args.body
                    .get_requested_sponsorship_paid_in_currency()
                    .clone(),
            ),
            requested_sponsor: Some(args.body.get_requested_sponsor().clone()),
            receiver_account: Some(args.body.get_receiver_account().clone()),
            supervisor: args.body.get_supervisor(),
            timeline: Some(serde_json::Value::from(args.body.get_timeline().clone())),
            views: None, // Last snapshot + 1
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

#[near(serializers=[borsh, json])]
#[derive(Clone)]
// NOTE: deserializing didn't work for some reason so instead we use get_proposal from RPC
struct EditProposalArgs {
    id: ProposalId,
    body: VersionedProposalBody,
    labels: HashSet<String>,
}

#[derive(Deserialize, Clone)]
struct PartialEditProposalArgs {
    id: i32,
}

#[derive(FromForm)]
struct ProposalQuery {
    limit: Option<usize>,
}

// add query params to get_proposals entrypoint
#[utoipa::path(get, path = "/proposals")]
#[get("/?<order>&<limit>&<offset>&<filtered_account_id>&<block_timestamp>")]
// Json<Proposal>
async fn get_proposals(
    order: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    filtered_account_id: Option<String>,
    block_timestamp: Option<i64>, // support for feed update functionality
    db: &State<DB>,
    // Json<PaginatedResponse<ProposalWithLatestSnapshotView>>
) -> Option<Json<PaginatedResponse<ProposalWithLatestSnapshotView>>> {
    // Get current timestamp
    // let current_timestamp = chrono::Utc::now().timestamp();
    let current_timestamp_nano = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    // Get last timestamp when database was updated
    let last_updated_timestamp = db.get_last_updated_timestamp().await.unwrap();

    println!("last_updated_timestamp: {:?}", last_updated_timestamp);
    println!("current_timestamp: {:?}", current_timestamp_nano);

    println!(
        "Difference: {:?}",
        current_timestamp_nano - last_updated_timestamp
    );
    println!(
        "Duration: {:?}",
        chrono::Duration::seconds(60).num_nanoseconds().unwrap()
    );
    // If we called nearblocks in the last 60 milliseconds return the database values
    if current_timestamp_nano - last_updated_timestamp
        < chrono::Duration::seconds(60).num_nanoseconds().unwrap()
    {
        let _proposals = db.get_proposals().await;
        println!("Returning cached proposals");
        return None;
    }

    println!("Fetching not yet indexed method calls from nearblocks");

    let nearblocks_client = nearblocks_client::ApiClient::default();

    // Nearblocks reacts with all contract changes since the timestamp we pass
    // This could return 0 new tx in which case we get the database stuff anyway
    // Or it could return 1 new tx in which case we want to update the database first
    // then get it from database using the right queries
    let nearblocks_unwrapped = match nearblocks_client
        .get_account_txns_by_pagination(
            "devhub.near".parse::<AccountId>().unwrap(),
            // Instead of just set_block_height_callback we should get all method calls
            // and handle them accordingly.
            Some("set_block_height_callback".to_string()),
            Some(timestamp_to_date_string(last_updated_timestamp)),
            // if this limit hits 10 we might need to do it in a loop let's say there are 100 changes since the last call to nearblocks.
            Some(25),
            Some("asc".to_string()),
        )
        .await
    {
        Ok(nearblocks_unwrapped) => {
            // If the response was successful, print the count of method calls
            // println!("Response: {:?}", nearblocks_unwrapped);
            nearblocks_unwrapped
        }
        Err(e) => {
            // If there was an error, print it or handle it as needed
            eprintln!("Failed to fetch data from nearblocks: {:?}", e);
            nearblocks_client::ApiResponse { txns: vec![] }
        }
    };

    println!(
        "Fetched {} method calls from nearblocks",
        nearblocks_unwrapped.clone().txns.len()
    );

    process_transactions(&nearblocks_unwrapped.txns, db).await;

    match nearblocks_unwrapped
        .txns
        // should we get the first or last?
        .last()
    {
        Some(transaction) => {
            println!("Added proposals to database, now adding timestamp.");

            println!("Transaction timestamp: {}", transaction.block_timestamp);
            let timestamp_nano: i64 = transaction.block_timestamp.parse().unwrap();

            println!("Parsed tx timestamp: {}", timestamp_nano);
            db.set_last_updated_timestamp(timestamp_nano).await.unwrap();

            println!("Added timestamp to database, returning proposals...");
        }
        None => {
            println!("No transactions found")
        }
    };

    // TODO add back in
    let order = order.unwrap_or("desc");
    let limit = limit.unwrap_or(25);
    let offset = offset.unwrap_or(0);
    // let block_timestamp = block_timestamp.unwrap_or(None);

    let proposals = match db
        .get_proposals_with_latest_snapshot(
            limit,
            order,
            offset,
            filtered_account_id,
            block_timestamp,
        )
        .await
    {
        Err(e) => {
            // race_of_sloths_server::error(
            //     telegram,
            //     &format!("Failed to get user contributions: {username}: {e}"),
            // );
            println!("Failed to get proposals: {:?}", e);
            return None;
        }
        Ok(proposals) => proposals,
    };

    Some(Json(PaginatedResponse::new(
        proposals.into_iter().map(Into::into).collect(),
        1,
        limit.try_into().unwrap(),
        0, // TODO TOTAL
    )))
}

async fn process_transactions(
    transactions: &Vec<Transaction>,
    db: &State<DB>,
) -> Result<String, Status> {
    for transaction in transactions.iter() {
        if let Some(action) = transaction.actions.get(0) {
            let result = match action.method.as_str() {
                "set_block_height_callback" => {
                    handle_set_block_height_callback(transaction.to_owned(), db).await
                }
                "edit_proposal_versioned_timeline" => {
                    handle_edit_proposal(transaction.to_owned(), db).await
                }
                "edit_proposal_timeline" => handle_edit_proposal(transaction.to_owned(), db).await,
                "edit_proposal" => handle_edit_proposal(transaction.to_owned(), db).await,
                "edit_proposal_linked_rfp" => {
                    handle_edit_proposal(transaction.to_owned(), db).await
                }
                _ => {
                    println!("Unhandled method: {}", action.method);
                    continue;
                } // or do something else if you want
            };
            if let Err(e) = result {
                return Err(e);
            }
        }
    }

    Ok("All transactions processed successfully".to_string())
}

async fn handle_set_block_height_callback(
    transaction: Transaction,
    db: &State<DB>,
) -> Result<String, Status> {
    let action = transaction.clone().actions.first().unwrap().clone();
    let json_args = action.args.clone();

    // println!("json_args: {:?}", json_args.clone());
    let args: SetBlockHeightCallbackArgs = serde_json::from_str(&json_args).unwrap();

    println!("Adding to the database... {}", args.clone().proposal.id);
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

    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok("ok".to_string())
}

fn get_proposal_id(transaction: &Transaction) -> i32 {
    // handle the unwrap better
    let action = transaction.clone().actions.first().unwrap().clone();
    let json_args = action.args.clone();

    let args: PartialEditProposalArgs = match serde_json::from_str(&json_args) {
        Ok(parsed_args) => parsed_args,
        Err(e) => {
            eprintln!("Failed to parse JSON: {:?}", e);
            PartialEditProposalArgs { id: 0 }
        }
    };
    args.id
}

async fn handle_edit_proposal(
    transaction: Transaction,
    db: &State<DB>,
) -> Result<String, rocket::http::Status> {
    let rpc_service = RpcService::default();
    let id = get_proposal_id(&transaction);
    let versioned_proposal = match rpc_service.get_proposal(id).await {
        Ok(proposal) => proposal,
        Err(e) => {
            eprintln!("Failed to get proposal from RPC: {:?}", e);
            return Err(Status::InternalServerError);
        }
    };

    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;

    let snapshot = ProposalSnapshotRecord::from_contract_proposal(
        versioned_proposal.into(),
        transaction.block_timestamp,
        transaction.block.block_height,
    );

    // Upsert into postgres
    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok("ok".to_string())
}

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

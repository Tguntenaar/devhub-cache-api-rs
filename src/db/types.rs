use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Proposal {
    pub id: i32,
    pub author_id: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct AfterDate {
    pub after_date: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct ProposalSnapshot {
    pub proposal_id: i32,
    pub block_height: i64,
    pub ts: i32,
    pub editor_id: String,
    pub social_db_post_block_height: i64,
    pub labels: serde_json::Value,
    pub proposal_version: String,
    pub proposal_body_version: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub linked_proposals: Option<serde_json::Value>,
    pub linked_rfp: Option<i32>,
    pub requested_sponsorship_usd_amount: Option<i32>,
    pub requested_sponsorship_paid_in_currency: Option<String>,
    pub requested_sponsor: Option<String>,
    pub receiver_account: Option<String>,
    pub supervisor: Option<String>,
    pub timeline: Option<serde_json::Value>,
    pub views: Option<i32>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Dump {
    pub receipt_id: String,
    pub method_name: String,
    pub block_height: i64,
    pub block_timestamp: i32,
    pub args: String,
    pub author: String,
    pub proposal_id: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct ProposalWithLatestSnapshot {
    pub proposal_id: i32,
    pub author_id: String,
    pub block_height: i64,
    pub ts: i32,
    pub editor_id: String,
    pub social_db_post_block_height: i64,
    pub labels: serde_json::Value,
    pub proposal_version: String,
    pub proposal_body_version: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub linked_proposals: Option<serde_json::Value>,
    pub linked_rfp: Option<i32>,
    pub requested_sponsorship_usd_amount: Option<i32>,
    pub requested_sponsorship_paid_in_currency: Option<String>,
    pub requested_sponsor: Option<String>,
    pub receiver_account: Option<String>,
    pub supervisor: Option<String>,
    pub timeline: Option<serde_json::Value>,
    pub views: Option<i32>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Rfp {
    pub id: i32,
    pub author_id: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct RfpSnapshot {
    pub rfp_id: i32,
    pub block_height: i64,
    pub ts: i32,
    pub editor_id: String,
    pub social_db_post_block_height: i64,
    pub labels: serde_json::Value,
    pub linked_proposals: Option<serde_json::Value>,
    pub rfp_version: String,
    pub rfp_body_version: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub timeline: Option<serde_json::Value>,
    pub submission_deadline: i32,
    pub views: Option<i32>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct RfpWithLatestSnapshot {
    pub rfp_id: i32,
    pub author_id: String,
    pub block_height: i64,
    pub ts: i32,
    pub editor_id: String,
    pub social_db_post_block_height: i64,
    pub labels: serde_json::Value,
    pub linked_proposals: Option<serde_json::Value>,
    pub rfp_version: String,
    pub rfp_body_version: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub timeline: Option<serde_json::Value>,
    pub views: Option<i32>,
    pub submission_deadline: i32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct RfpDump {
    pub receipt_id: String,
    pub method_name: String,
    pub block_height: i64,
    pub block_timestamp: i32,
    pub args: String,
    pub author: String,
    pub rfp_id: i64,
}

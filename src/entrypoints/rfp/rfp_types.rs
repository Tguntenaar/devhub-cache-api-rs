use crate::db::db_types::RfpSnapshotRecord;
use devhub_shared::rfp::RFP as ContractRFP;
use devhub_shared::rfp::{VersionedRFPBody, RFP};
use rocket::serde::{Deserialize, Serialize};
use rocket::FromForm;
use utoipa::ToSchema;

#[derive(Deserialize)]
pub struct PartialEditRFPArgs {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct PartialEditRFPTimelineArgs {
    pub id: i32,
    pub timeline: String,
}

#[derive(Clone, Debug, FromForm, ToSchema)]
pub struct GetRfpFilters {
    pub category: Option<String>,
    pub labels: Option<Vec<String>>,
    pub input: Option<String>,
    pub author_id: Option<String>,
    pub stage: Option<String>,
    pub block_timestamp: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SetRfpBlockHeightCallbackArgs {
    pub rfp: RFP,
}

pub trait FromContractProposal {
    fn from_contract_rfp(rfp: ContractRFP, timestamp: String, block_height: i64) -> Self;
}

pub trait FromContractRFP {
    fn from_contract_rfp(rfp: ContractRFP, timestamp: String, block_height: i64) -> Self;
}

impl FromContractRFP for RfpSnapshotRecord {
    fn from_contract_rfp(rfp: ContractRFP, timestamp: String, block_height: i64) -> Self {
        RfpSnapshotRecord {
            rfp_id: rfp.id as i32,
            block_height,
            ts: timestamp.parse::<i64>().unwrap_or_default(),
            editor_id: rfp.snapshot.editor_id.to_string(),
            social_db_post_block_height: rfp.social_db_post_block_height as i64,
            labels: serde_json::Value::from(Vec::from_iter(rfp.snapshot.labels.iter().cloned())),
            linked_proposals: Some(serde_json::Value::from(Vec::from_iter(
                rfp.snapshot.linked_proposals,
            ))),
            rfp_version: "V0".to_string(),
            rfp_body_version: "V0".to_string(),
            name: Some(rfp.snapshot.body.get_name().clone()),
            category: None,
            summary: Some(rfp.snapshot.body.get_summary().clone()),
            description: Some(rfp.snapshot.body.get_description().clone()),
            timeline: Some(serde_json::Value::from(
                rfp.snapshot.body.get_timeline().clone(),
            )),
            submission_deadline: rfp.snapshot.body.get_submission_deadline(),
            views: Some(0),
        }
    }
}

pub trait RfpBodyFields {
    fn get_name(&self) -> &String;
    fn get_summary(&self) -> &String;
    fn get_description(&self) -> &String;
    fn get_timeline(&self) -> String;
    fn get_submission_deadline(&self) -> i64;
}

impl RfpBodyFields for VersionedRFPBody {
    fn get_name(&self) -> &String {
        match self {
            VersionedRFPBody::V0(body) => &body.name,
        }
    }

    fn get_summary(&self) -> &String {
        match self {
            VersionedRFPBody::V0(body) => &body.summary,
        }
    }

    fn get_description(&self) -> &String {
        match self {
            VersionedRFPBody::V0(body) => &body.description,
        }
    }

    fn get_timeline(&self) -> String {
        match self {
            VersionedRFPBody::V0(body) => serde_json::to_string(&body.timeline).unwrap_or_default(),
        }
    }

    fn get_submission_deadline(&self) -> i64 {
        match self {
            VersionedRFPBody::V0(body) => body.submission_deadline.try_into().unwrap_or_default(),
        }
    }
}

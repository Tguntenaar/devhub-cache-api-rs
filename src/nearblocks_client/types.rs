use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::db_types::BlockHeight;

// The RPC is sometimes not yet updated with the at the receipt block height, so we add an offset to ensure latest version
pub const BLOCK_HEIGHT_OFFSET: i64 = 10;
pub struct LinkedProposals(pub Vec<i32>);

impl From<Option<Value>> for LinkedProposals {
    fn from(value: Option<Value>) -> Self {
        if let Some(Value::Array(arr)) = value {
            let vec = arr
                .into_iter()
                .filter_map(|v| v.as_i64().map(|n| n as i32))
                .collect();
            LinkedProposals(vec)
        } else {
            LinkedProposals(Vec::new())
        }
    }
}

impl From<LinkedProposals> for Option<Value> {
    fn from(linked_proposals: LinkedProposals) -> Self {
        Some(Value::Array(
            linked_proposals
                .0
                .into_iter()
                .map(|n| Value::Number(n.into()))
                .collect(),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    #[serde(default)]
    pub id: String,
    pub receipt_id: String,
    pub predecessor_account_id: String,
    pub receiver_account_id: String,
    pub receipt_kind: String,
    pub receipt_block: Block,
    pub receipt_outcome: ReceiptOutcome,
    pub transaction_hash: String,
    pub included_in_block_hash: String,
    pub block_timestamp: String,
    pub block: BlockInfo,
    pub receipt_conversion_tokens_burnt: String,
    pub actions: Option<Vec<Action>>,
    pub actions_agg: ActionsAgg,
    pub outcomes: Outcomes,
    pub outcomes_agg: OutcomesAgg,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub block_hash: String,
    pub block_height: BlockHeight,
    pub block_timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReceiptOutcome {
    pub gas_burnt: f64,
    pub tokens_burnt: f64,
    pub executor_account_id: String,
    pub status: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockInfo {
    pub block_height: BlockHeight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub action: String,
    #[serde(default)]
    pub method: Option<String>,
    pub deposit: f64,
    pub fee: f64,
    #[serde(default)]
    pub args: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionsAgg {
    pub deposit: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Outcomes {
    pub status: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutcomesAgg {
    pub transaction_fee: f64,
}

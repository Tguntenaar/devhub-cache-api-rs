use serde::{Deserialize, Serialize};

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
    pub block_height: f64,
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
    pub block_height: i64,
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

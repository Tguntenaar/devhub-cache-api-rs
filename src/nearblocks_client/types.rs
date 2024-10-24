use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Transaction {
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
    pub actions: Vec<Action>,
    pub actions_agg: ActionsAgg,
    pub outcomes: Outcomes,
    pub outcomes_agg: OutcomesAgg,
}

#[derive(Deserialize, Debug)]
pub struct Block {
    pub block_hash: String,
    pub block_height: i64,
    pub block_timestamp: i64,
}

#[derive(Deserialize, Debug)]
pub struct ReceiptOutcome {
    pub gas_burnt: f64,
    pub tokens_burnt: f64,
    pub executor_account_id: String,
    pub status: bool,
}

#[derive(Deserialize, Debug)]
pub struct BlockInfo {
    pub block_height: i64,
}

#[derive(Deserialize, Debug)]
pub struct Action {
    pub action: String,
    pub method: String,
    pub deposit: i64,
    pub fee: f64,
    pub args: String,
}

#[derive(Deserialize, Debug)]
pub struct ActionsAgg {
    pub deposit: i64,
}

#[derive(Deserialize, Debug)]
pub struct Outcomes {
    pub status: bool,
}

#[derive(Deserialize, Debug)]
pub struct OutcomesAgg {
    pub transaction_fee: f64,
}

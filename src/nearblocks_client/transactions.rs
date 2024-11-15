use crate::db::db_types::ProposalSnapshotRecord;
use crate::db::db_types::RfpSnapshotRecord;
use crate::db::DB;
use crate::entrypoints::proposal::proposal_types::FromContractProposal;
use crate::entrypoints::proposal::proposal_types::PartialEditProposalArgs;
use crate::entrypoints::proposal::proposal_types::SetBlockHeightCallbackArgs;
use crate::entrypoints::rfp::rfp_types::*;
use crate::nearblocks_client::types::Transaction;
use crate::rpc_service::RpcService;
use devhub_shared::proposal::VersionedProposal;
use devhub_shared::rfp::VersionedRFP;
use near_account_id::AccountId;
use rocket::{http::Status, State};
use std::convert::TryInto;

pub async fn process(
    transactions: &[Transaction],
    db: &State<DB>,
    contract: AccountId,
) -> Result<(), Status> {
    for transaction in transactions.iter() {
        if let Some(action) = transaction
            .actions
            .as_ref()
            .and_then(|actions| actions.first())
        {
            let result = match action.method.as_deref().unwrap_or("") {
                "set_block_height_callback" => {
                    handle_set_block_height_callback(transaction.to_owned(), db, contract.clone())
                        .await
                }
                "edit_proposal" => {
                    handle_edit_proposal(transaction.to_owned(), db, contract.clone()).await
                }
                "edit_proposal_timeline" => {
                    handle_edit_proposal(transaction.to_owned(), db, contract.clone()).await
                }
                "edit_proposal_versioned_timeline" => {
                    handle_edit_proposal(transaction.to_owned(), db, contract.clone()).await
                }
                "edit_proposal_linked_rfp" => {
                    handle_edit_proposal(transaction.to_owned(), db, contract.clone()).await
                }
                "edit_proposal_internal" => {
                    handle_edit_proposal(transaction.to_owned(), db, contract.clone()).await
                }
                "edit_rfp_timeline" => {
                    handle_edit_rfp(transaction.to_owned(), db, contract.clone()).await
                }
                "edit_rfp" => handle_edit_rfp(transaction.to_owned(), db, contract.clone()).await,
                "edit_rfp_internal" => {
                    handle_edit_rfp(transaction.to_owned(), db, contract.clone()).await
                }
                "cancel_rfp" => handle_edit_rfp(transaction.to_owned(), db, contract.clone()).await,
                "set_rfp_block_height_callback" => {
                    handle_set_rfp_block_height_callback(
                        transaction.to_owned(),
                        db,
                        contract.clone(),
                    )
                    .await
                }
                _ => {
                    // println!("Unhandled method: {:?}", action.method);
                    continue;
                }
            };
            result?;
        }
    }

    Ok(())
}

async fn handle_set_rfp_block_height_callback(
    transaction: Transaction,
    db: &State<DB>,
    contract: AccountId,
) -> Result<(), Status> {
    let action = transaction
        .actions
        .as_ref()
        .and_then(|actions| actions.first())
        .ok_or(Status::InternalServerError)?;
    let json_args = action.args.clone().unwrap_or_default();

    // println!("json_args: {:?}", json_args.clone());
    let args: SetRfpBlockHeightCallbackArgs = serde_json::from_str(&json_args).unwrap();

    println!("Adding to the database... {}", args.clone().rfp.id);
    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;
    DB::upsert_rfp(
        &mut tx,
        args.clone().rfp.id,
        args.clone().rfp.author_id.to_string(),
    )
    .await
    .unwrap();

    let rpc_service = RpcService::new(contract);
    let id = args.clone().rfp.id.try_into().unwrap();

    println!("stored rfp {}", id);

    let versioned_rfp_fallback: VersionedRFP = args.clone().rfp.into();
    let versioned_rfp = match rpc_service.get_rfp(id).await {
        Ok(rfp) => rfp.data,
        Err(e) => {
            eprintln!(
                "Failed to get RFP from RPC, using first snapshot as fallback {:?}",
                e
            );
            versioned_rfp_fallback
        }
    };

    let snapshot = RfpSnapshotRecord::from_contract_rfp(
        versioned_rfp.into(),
        transaction.block_timestamp,
        transaction.block.block_height as i64,
    );

    DB::insert_rfp_snapshot(&mut tx, &snapshot).await.unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok(())
}

fn get_rfp_id(transaction: &Transaction) -> Result<i32, &'static str> {
    let action = transaction
        .actions
        .as_ref()
        .and_then(|actions| actions.first())
        .ok_or("No actions found in transaction")?;

    let args: PartialEditRFPArgs =
        serde_json::from_str(&action.args.as_ref().unwrap()).map_err(|e| {
            eprintln!("Failed to parse JSON: {:?}", e);
            "Failed to parse proposal arguments"
        })?;

    Ok(args.id)
}

async fn handle_edit_rfp(
    transaction: Transaction,
    db: &State<DB>,
    contract: AccountId,
) -> Result<(), Status> {
    let rpc_service = RpcService::new(contract);
    let id = get_rfp_id(&transaction).map_err(|e| {
        eprintln!("Failed to get RFP ID: {}", e);
        Status::InternalServerError
    })?;
    println!("Updating rfp {}", id);
    let versioned_rfp = match rpc_service.get_rfp(id).await {
        Ok(rfp) => rfp.data,
        Err(e) => {
            eprintln!("Failed to get rfp from RPC: {:?}", e);
            return Err(Status::InternalServerError);
        }
    };

    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;

    let snapshot = RfpSnapshotRecord::from_contract_rfp(
        versioned_rfp.into(),
        transaction.block_timestamp,
        transaction.block.block_height as i64,
    );

    DB::insert_rfp_snapshot(&mut tx, &snapshot)
        .await
        .map_err(|_e| Status::InternalServerError)?;

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok(())
}

async fn handle_set_block_height_callback(
    transaction: Transaction,
    db: &State<DB>,
    contract: AccountId,
) -> Result<(), Status> {
    let action = transaction
        .actions
        .as_ref()
        .and_then(|actions| actions.first())
        .ok_or(Status::InternalServerError)?;

    let json_args = action.args.clone();

    let args: SetBlockHeightCallbackArgs =
        serde_json::from_str(&json_args.unwrap_or_default()).unwrap();

    println!("Adding to the database... {}", args.clone().proposal.id);
    // TODO move txs the the outside
    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;
    DB::upsert_proposal(
        &mut tx,
        args.clone().proposal.id,
        args.clone().proposal.author_id.to_string(),
    )
    .await
    .unwrap();

    let rpc_service = RpcService::new(contract);
    let id = args.clone().proposal.id.try_into().unwrap();

    println!("stored proposal {}", id);

    let versioned_proposal_fallback: VersionedProposal = args.clone().proposal.into();
    let versioned_proposal = match rpc_service.get_proposal(id).await {
        Ok(proposal) => proposal.data,
        Err(e) => {
            eprintln!(
                "Failed to get proposal from RPC, using first snapshot as fallback {:?}",
                e
            );
            versioned_proposal_fallback
        }
    };

    let snapshot = ProposalSnapshotRecord::from_contract_proposal(
        versioned_proposal.into(),
        transaction.block_timestamp,
        transaction.block.block_height as i64,
    );

    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok(())
}

fn get_proposal_id(transaction: &Transaction) -> Result<i32, &'static str> {
    let action = transaction
        .actions
        .as_ref()
        .and_then(|actions| actions.first())
        .ok_or("No actions found in transaction")?;

    let args: PartialEditProposalArgs = serde_json::from_str(&action.args.as_ref().unwrap())
        .map_err(|e| {
            eprintln!("Failed to parse JSON: {:?}", e);
            "Failed to parse proposal arguments"
        })?;

    Ok(args.id)
}

async fn handle_edit_proposal(
    transaction: Transaction,
    db: &State<DB>,
    contract: AccountId,
) -> Result<(), rocket::http::Status> {
    let rpc_service = RpcService::new(contract);
    let id = get_proposal_id(&transaction).map_err(|e| {
        eprintln!("Failed to get proposal ID: {}", e);
        Status::InternalServerError
    })?;
    println!("Updating proposal {}", id);
    let versioned_proposal = match rpc_service.get_proposal(id).await {
        Ok(proposal) => proposal.data,
        Err(e) => {
            eprintln!("Failed to get proposal from RPC: {:?}", e);
            return Err(Status::InternalServerError);
        }
    };

    let mut tx = db.begin().await.map_err(|_e| Status::InternalServerError)?;

    let snapshot = ProposalSnapshotRecord::from_contract_proposal(
        versioned_proposal.into(),
        transaction.block_timestamp,
        transaction.block.block_height as i64,
    );

    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .unwrap();

    tx.commit()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    Ok(())
}

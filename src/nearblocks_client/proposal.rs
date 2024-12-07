use crate::db::db_types::{BlockHeight, ProposalSnapshotRecord, Timestamp};
use crate::db::DB;
use crate::entrypoints::proposal::proposal_types::{
    FromContractProposal, PartialEditProposalArgs, SetBlockHeightCallbackArgs,
};
use crate::nearblocks_client::types::{LinkedProposals, Transaction, BLOCK_HEIGHT_OFFSET};
use crate::rpc_service::RpcService;
use devhub_shared::proposal::VersionedProposal;
use near_account_id::AccountId;
use rocket::{http::Status, State};

pub async fn handle_set_block_height_callback(
    transaction: Transaction,
    db: &State<DB>,
    contract: &AccountId,
) -> Result<(), Status> {
    let action = transaction
        .actions
        .as_ref()
        .and_then(|actions| actions.first())
        .ok_or(Status::InternalServerError)?;

    let json_args = action.args.clone();

    let args: SetBlockHeightCallbackArgs =
        serde_json::from_str(&json_args.unwrap_or_default()).unwrap();

    let mut tx = db.begin().await.map_err(|e| {
        eprintln!("Failed to begin transaction: {:?}", e);
        Status::InternalServerError
    })?;
    DB::upsert_proposal(
        &mut tx,
        args.clone().proposal.id,
        args.clone().proposal.author_id.to_string(),
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to upsert proposal {}: {:?}", args.proposal.id, e);
        Status::InternalServerError
    })?;

    let rpc_service = RpcService::new(contract);
    let id = args.clone().proposal.id.try_into().unwrap();

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
        versioned_proposal.clone().into(),
        transaction.block_timestamp.parse::<i64>().unwrap(),
        transaction.block.block_height,
    );

    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .map_err(|e| {
            eprintln!(
                "Failed to insert proposal snapshot for proposal {}: {:?}",
                id, e
            );
            Status::InternalServerError
        })?;

    tx.commit().await.map_err(|e| {
        eprintln!("Failed to commit transaction: {:?}", e);
        Status::InternalServerError
    })?;

    Ok(())
}

pub async fn handle_edit_proposal(
    transaction: Transaction,
    db: &State<DB>,
    contract: &AccountId,
) -> Result<(), rocket::http::Status> {
    let rpc_service = RpcService::new(contract);
    let id = get_proposal_id(&transaction).map_err(|e| {
        eprintln!("Failed to get proposal ID: {}", e);
        Status::InternalServerError
    })?;
    println!("Updating proposal {}", id);
    let versioned_proposal = match rpc_service
        .get_proposal_on_block(
            id,
            transaction.receipt_block.block_height + BLOCK_HEIGHT_OFFSET,
        )
        .await
    {
        Ok(proposal) => proposal,
        Err(e) => {
            eprintln!("Failed to get proposal from RPC: {:?}", e);
            return Err(Status::InternalServerError);
        }
    };

    let mut tx = db.begin().await.map_err(|e| {
        eprintln!("Failed to begin transaction: {:?}", e);
        Status::InternalServerError
    })?;

    let snapshot = ProposalSnapshotRecord::from_contract_proposal(
        versioned_proposal.clone().into(),
        transaction.block_timestamp.parse::<i64>().unwrap(),
        transaction.block.block_height,
    );

    DB::insert_proposal_snapshot(&mut tx, &snapshot)
        .await
        .map_err(|e| {
            eprintln!(
                "Failed to insert proposal snapshot for proposal {}: {:?}",
                id, e
            );
            Status::InternalServerError
        })?;

    tx.commit().await.map_err(|e| {
        eprintln!("Failed to commit transaction: {:?}", e);
        Status::InternalServerError
    })?;

    Ok(())
}

fn get_proposal_id(transaction: &Transaction) -> Result<i32, &'static str> {
    let action = transaction
        .actions
        .as_ref()
        .and_then(|actions| actions.first())
        .ok_or("No actions found in transaction")?;

    let args: PartialEditProposalArgs = serde_json::from_str(action.args.as_ref().unwrap())
        .map_err(|e| {
            eprintln!("Failed to parse JSON: {:?}", e);
            "Failed to parse proposal arguments"
        })?;

    Ok(args.id)
}

// This might not be needed since we are getting the proposal snapshot from the RPC on the block height.
pub async fn update_linked_proposals(
    proposal_id: i32,
    new_linked_rfp: Option<i32>,
    block_height: BlockHeight,
    block_timestamp: Timestamp,
    db: &State<DB>,
) -> Result<(), Status> {
    // Get the latest proposal snapshot before the given block timestamp
    let last_snapshot = db
        .get_proposal_with_latest_snapshot_view(proposal_id)
        .await
        .map_err(|e| {
            eprintln!(
                "Error fetching latest proposal snapshot for proposal {}: {:?}",
                proposal_id, e
            );
            Status::InternalServerError
        })?;

    // Extract the latest linked RFP ID from the snapshot
    let latest_linked_rfp_id = last_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.linked_rfp);

    // Compare the new and old linked RFP IDs
    if new_linked_rfp != latest_linked_rfp_id {
        let tx = db.begin().await.map_err(|e| {
            eprintln!("Failed to begin transaction: {:?}", e);
            Status::InternalServerError
        })?;

        if let Some(new_linked_rfp_id) = new_linked_rfp {
            println!(
                "Adding linked_rfp {} to proposal {}",
                new_linked_rfp_id, proposal_id
            );

            // Add linked proposal to new RFP snapshot
            modify_snapshot_linked_proposal(
                new_linked_rfp_id,
                proposal_id,
                block_height,
                block_timestamp,
                add_to_linked_proposals,
                db,
            )
            .await?;
            println!("Proposal added to new RFP snapshot");
        }

        if let Some(old_linked_rfp_id) = latest_linked_rfp_id {
            println!(
                "Removing linked_rfp {} from proposal {}",
                old_linked_rfp_id, proposal_id
            );
            // Remove linked proposal from old RFP snapshot
            modify_snapshot_linked_proposal(
                old_linked_rfp_id,
                proposal_id,
                block_height,
                block_timestamp,
                remove_from_linked_proposals,
                db,
            )
            .await?;
            println!("Proposal removed from old RFP snapshot");
        }

        tx.commit().await.map_err(|e| {
            eprintln!("Failed to commit transaction: {:?}", e);
            Status::InternalServerError
        })?;
    }

    Ok(())
}

fn add_to_linked_proposals(mut linked_proposals: Vec<i32>, proposal_id: i32) -> Vec<i32> {
    linked_proposals.push(proposal_id);
    linked_proposals
}

fn remove_from_linked_proposals(linked_proposals: Vec<i32>, proposal_id: i32) -> Vec<i32> {
    linked_proposals
        .into_iter()
        .filter(|&id| id != proposal_id)
        .collect()
}

async fn modify_snapshot_linked_proposal(
    rfp_id: i32,
    proposal_id: i32,
    block_height: BlockHeight,
    block_timestamp: Timestamp,
    callback: fn(Vec<i32>, i32) -> Vec<i32>,
    db: &State<DB>,
) -> Result<(), Status> {
    let latest_rfp_snapshot = db.get_latest_rfp_snapshot(rfp_id).await.map_err(|e| {
        eprintln!(
            "Failed to get latest RFP snapshot for RFP {}: {:?}",
            rfp_id, e
        );
        Status::InternalServerError
    })?;

    if let Some(mut snapshot) = latest_rfp_snapshot {
        // Update the snapshot with new values
        snapshot.rfp_id = rfp_id;
        let linked_proposals: LinkedProposals = snapshot.linked_proposals.into();
        let updated_proposals = callback(linked_proposals.0, proposal_id);
        snapshot.linked_proposals = LinkedProposals(updated_proposals).into();
        snapshot.block_height = block_height;
        snapshot.ts = block_timestamp;

        let mut tx = db.begin().await.map_err(|e| {
            eprintln!("Failed to begin transaction: {:?}", e);
            Status::InternalServerError
        })?;

        DB::insert_rfp_snapshot(&mut tx, &snapshot)
            .await
            .map_err(|e| {
                eprintln!("Failed to insert RFP snapshot for RFP {}: {:?}", rfp_id, e);
                Status::InternalServerError
            })?;

        tx.commit().await.map_err(|e| {
            eprintln!("Failed to commit transaction: {:?}", e);
            Status::InternalServerError
        })?;
    } else {
        eprintln!("No existing RFP snapshot found for RFP {}", rfp_id);
    }

    Ok(())
}

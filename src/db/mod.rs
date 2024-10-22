use rocket::{
    fairing::{self, AdHoc},
    Build, Rocket,
};
use rocket_db_pools::Database;
use sqlx::{PgPool, Postgres, Transaction};
#[derive(Database, Clone, Debug)]
#[database("devhub_cache")] // Adjust the database name accordingly
pub struct DB(PgPool);

pub mod types;

use types::{
    Dump, Proposal, ProposalSnapshot, ProposalWithLatestSnapshot, RfpDump, RfpSnapshot,
    RfpWithLatestSnapshot,
};

impl DB {
    // Functions for Proposals
    pub async fn insert_proposal(
        tx: &mut Transaction<'static, Postgres>,
        author_id: String,
    ) -> Result<i32, sqlx::Error> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO proposals (author_id)
            VALUES ($1)
            RETURNING id
            "#,
            author_id
        )
        .fetch_optional(tx.as_mut()) // Use `fetch_one` instead of `execute`
        .await?;

        if let Some(record) = rec {
            Ok(record.id)
        } else {
            let rec = sqlx::query!(
                r#"
            SELECT id FROM proposals
            WHERE author_id = $1
            "#,
                author_id
            )
            .fetch_one(tx.as_mut())
            .await?;

            Ok(rec.id)
        }
    }

    //     // pub async fn get_proposal_by_id(
    //     //     tx: &mut Transaction<'static, Postgres>,
    //     //     proposal_id: i32,
    //     // ) -> anyhow::Result<Option<Proposal>> {
    //     //     let rec = sqlx::query_as!(
    //     //         Proposal,
    //     //         r#"
    //     //       SELECT id, author_id
    //     //       FROM proposals
    //     //       WHERE id = $1
    //     //       "#,
    //     //         proposal_id
    //     //     )
    //     //     .fetch_optional(tx)
    //     //     .await?;
    //     //     Ok(rec)
    //     // }

    //     // pub async fn upsert_proposal_snapshot(
    //     //     tx: &mut Transaction<'static, Postgres>,
    //     //     snapshot: &ProposalSnapshot,
    //     // ) -> anyhow::Result<()> {
    //     //     // Since primary key is (proposal_id, ts)
    //     //     sqlx::query!(
    //     //         r#"
    //     //       INSERT INTO proposal_snapshots (
    //     //           proposal_id,
    //     //           block_height,
    //     //           ts,
    //     //           editor_id,
    //     //           social_db_post_block_height,
    //     //           labels,
    //     //           proposal_version,
    //     //           proposal_body_version,
    //     //           name,
    //     //           category,
    //     //           summary,
    //     //           description,
    //     //           linked_proposals,
    //     //           linked_rfp,
    //     //           requested_sponsorship_usd_amount,
    //     //           requested_sponsorship_paid_in_currency,
    //     //           requested_sponsor,
    //     //           receiver_account,
    //     //           supervisor,
    //     //           timeline,
    //     //           views
    //     //       ) VALUES (
    //     //           $1, $2, $3, $4, $5, $6, $7, $8,
    //     //           $9, $10, $11, $12, $13, $14,
    //     //           $15, $16, $17, $18, $19, $20, $21
    //     //       ) ON CONFLICT (proposal_id, ts) DO UPDATE SET
    //     //           block_height = $2,
    //     //           editor_id = $4,
    //     //           social_db_post_block_height = $5,
    //     //           labels = $6,
    //     //           proposal_version = $7,
    //     //           proposal_body_version = $8,
    //     //           name = $9,
    //     //           category = $10,
    //     //           summary = $11,
    //     //           description = $12,
    //     //           linked_proposals = $13,
    //     //           linked_rfp = $14,
    //     //           requested_sponsorship_usd_amount = $15,
    //     //           requested_sponsorship_paid_in_currency = $16,
    //     //           requested_sponsor = $17,
    //     //           receiver_account = $18,
    //     //           supervisor = $19,
    //     //           timeline = $20,
    //     //           views = $21
    //     //       "#,
    //     //         snapshot.proposal_id,
    //     //         snapshot.block_height,
    //     //         snapshot.ts,
    //     //         snapshot.editor_id,
    //     //         snapshot.social_db_post_block_height,
    //     //         snapshot.labels,
    //     //         snapshot.proposal_version,
    //     //         snapshot.proposal_body_version,
    //     //         snapshot.name,
    //     //         snapshot.category,
    //     //         snapshot.summary,
    //     //         snapshot.description,
    //     //         snapshot.linked_proposals,
    //     //         snapshot.linked_rfp,
    //     //         snapshot.requested_sponsorship_usd_amount,
    //     //         snapshot.requested_sponsorship_paid_in_currency,
    //     //         snapshot.requested_sponsor,
    //     //         snapshot.receiver_account,
    //     //         snapshot.supervisor,
    //     //         snapshot.timeline,
    //     //         snapshot.views
    //     //     )
    //     //     .execute(tx)
    //     //     .await?;
    //     //     Ok(())
    //     // }

    //     // pub async fn get_latest_proposal_snapshot(
    //     //     tx: &mut Transaction<'static, Postgres>,
    //     //     proposal_id: i32,
    //     // ) -> anyhow::Result<Option<ProposalSnapshot>> {
    //     //     let rec = sqlx::query_as!(
    //     //         ProposalSnapshot,
    //     //         r#"
    //     //       SELECT * FROM proposal_snapshots
    //     //       WHERE proposal_id = $1
    //     //       ORDER BY ts DESC
    //     //       LIMIT 1
    //     //       "#,
    //     //         proposal_id
    //     //     )
    //     //     .fetch_optional(tx)
    //     //     .await?;
    //     //     Ok(rec)
    //     // }

    //     // pub async fn insert_dump(
    //     //     tx: &mut Transaction<'static, Postgres>,
    //     //     dump: &Dump,
    //     // ) -> anyhow::Result<()> {
    //     //     sqlx::query!(
    //     //         r#"
    //     //       INSERT INTO dumps (
    //     //           receipt_id,
    //     //           method_name,
    //     //           block_height,
    //     //           block_timestamp,
    //     //           args,
    //     //           author,
    //     //           proposal_id
    //     //       ) VALUES (
    //     //           $1, $2, $3, $4, $5, $6, $7
    //     //       ) ON CONFLICT (receipt_id) DO NOTHING
    //     //       "#,
    //     //         dump.receipt_id,
    //     //         dump.method_name,
    //     //         dump.block_height,
    //     //         dump.block_timestamp,
    //     //         dump.args,
    //     //         dump.author,
    //     //         dump.proposal_id
    //     //     )
    //     //     .execute(tx)
    //     //     .await?;
    //     //     Ok(())
    //     // }

    //     // // Functions for RFPs

    //     // pub async fn insert_rfp(
    //     //     tx: &mut Transaction<'static, Postgres>,
    //     //     author_id: &str,
    //     // ) -> anyhow::Result<i32> {
    //     //     let rec = sqlx::query!(
    //     //         r#"
    //     //       INSERT INTO rfps (author_id)
    //     //       VALUES ($1)
    //     //       RETURNING id
    //     //       "#,
    //     //         author_id
    //     //     )
    //     //     .fetch_one(tx)
    //     //     .await?;

    //     //     Ok(rec.id)
    //     // }

    //     // pub async fn upsert_rfp_snapshot(
    //     //     tx: &mut Transaction<'static, Postgres>,
    //     //     snapshot: &RfpSnapshot,
    //     // ) -> anyhow::Result<()> {
    //     //     // Primary key is (rfp_id, ts)
    //     //     sqlx::query!(
    //     //         r#"
    //     //       INSERT INTO rfp_snapshots (
    //     //           rfp_id,
    //     //           block_height,
    //     //           ts,
    //     //           editor_id,
    //     //           social_db_post_block_height,
    //     //           labels,
    //     //           linked_proposals,
    //     //           rfp_version,
    //     //           rfp_body_version,
    //     //           name,
    //     //           category,
    //     //           summary,
    //     //           description,
    //     //           timeline,
    //     //           submission_deadline,
    //     //           views
    //     //       ) VALUES (
    //     //           $1, $2, $3, $4, $5, $6, $7, $8,
    //     //           $9, $10, $11, $12, $13, $14, $15, $16
    //     //       ) ON CONFLICT (rfp_id, ts) DO UPDATE SET
    //     //           block_height = $2,
    //     //           editor_id = $4,
    //     //           social_db_post_block_height = $5,
    //     //           labels = $6,
    //     //           linked_proposals = $7,
    //     //           rfp_version = $8,
    //     //           rfp_body_version = $9,
    //     //           name = $10,
    //     //           category = $11,
    //     //           summary = $12,
    //     //           description = $13,
    //     //           timeline = $14,
    //     //           submission_deadline = $15,
    //     //           views = $16
    //     //       "#,
    //     //         snapshot.rfp_id,
    //     //         snapshot.block_height,
    //     //         snapshot.ts,
    //     //         snapshot.editor_id,
    //     //         snapshot.social_db_post_block_height,
    //     //         snapshot.labels,
    //     //         snapshot.linked_proposals,
    //     //         snapshot.rfp_version,
    //     //         snapshot.rfp_body_version,
    //     //         snapshot.name,
    //     //         snapshot.category,
    //     //         snapshot.summary,
    //     //         snapshot.description,
    //     //         snapshot.timeline,
    //     //         snapshot.submission_deadline,
    //     //         snapshot.views
    //     //     )
    //     //     .execute(tx)
    //     //     .await?;
    //     //     Ok(())
    //     // }

    //     // pub async fn insert_rfp_dump(
    //     //     tx: &mut Transaction<'static, Postgres>,
    //     //     dump: &RfpDump,
    //     // ) -> anyhow::Result<()> {
    //     //     sqlx::query!(
    //     //         r#"
    //     //       INSERT INTO rfp_dumps (
    //     //           receipt_id,
    //     //           method_name,
    //     //           block_height,
    //     //           block_timestamp,
    //     //           args,
    //     //           author,
    //     //           rfp_id
    //     //       ) VALUES (
    //     //           $1, $2, $3, $4, $5, $6, $7
    //     //       ) ON CONFLICT (receipt_id) DO NOTHING
    //     //       "#,
    //     //         dump.receipt_id,
    //     //         dump.method_name,
    //     //         dump.block_height,
    //     //         dump.block_timestamp,
    //     //         dump.args,
    //     //         dump.author,
    //     //         dump.rfp_id
    //     //     )
    //     //     .execute(tx)
    //     //     .await?;
    //     //     Ok(())
    //     // }

    //     // // Function to get proposals with the latest snapshot

    //     // pub async fn get_proposals_with_latest_snapshot(
    //     //     &self,
    //     // ) -> anyhow::Result<Vec<ProposalWithLatestSnapshot>> {
    //     //     let recs = sqlx::query_as!(
    //     //         ProposalWithLatestSnapshot,
    //     //         r#"
    //     //       SELECT
    //     //         ps.proposal_id,
    //     //         p.author_id,
    //     //         ps.block_height,
    //     //         ps.ts,
    //     //         ps.editor_id,
    //     //         ps.social_db_post_block_height,
    //     //         ps.labels,
    //     //         ps.proposal_version,
    //     //         ps.proposal_body_version,
    //     //         ps.name,
    //     //         ps.category,
    //     //         ps.summary,
    //     //         ps.description,
    //     //         ps.linked_proposals,
    //     //         ps.linked_rfp,
    //     //         ps.requested_sponsorship_usd_amount,
    //     //         ps.requested_sponsorship_paid_in_currency,
    //     //         ps.requested_sponsor,
    //     //         ps.receiver_account,
    //     //         ps.supervisor,
    //     //         ps.timeline,
    //     //         ps.views
    //     //       FROM
    //     //         proposals p
    //     //         INNER JOIN (
    //     //           SELECT
    //     //             proposal_id,
    //     //             MAX(ts) AS max_ts
    //     //           FROM
    //     //             proposal_snapshots
    //     //           GROUP BY
    //     //             proposal_id
    //     //         ) latest_snapshots ON p.id = latest_snapshots.proposal_id
    //     //         INNER JOIN proposal_snapshots ps ON latest_snapshots.proposal_id = ps.proposal_id
    //     //         AND latest_snapshots.max_ts = ps.ts;
    //     //       "#
    //     //     )
    //     //     .fetch_all(&self.0)
    //     //     .await?;
    //     //     Ok(recs)
    //     // }

    //     // // Function to get RFPs with the latest snapshot

    //     // pub async fn get_rfps_with_latest_snapshot(
    //     //     &self,
    //     // ) -> anyhow::Result<Vec<RfpWithLatestSnapshot>> {
    //     //     let recs = sqlx::query_as!(
    //     //         RfpWithLatestSnapshot,
    //     //         r#"
    //     //       SELECT
    //     //         ps.rfp_id,
    //     //         p.author_id,
    //     //         ps.block_height,
    //     //         ps.ts,
    //     //         ps.editor_id,
    //     //         ps.social_db_post_block_height,
    //     //         ps.labels,
    //     //         ps.linked_proposals,
    //     //         ps.rfp_version,
    //     //         ps.rfp_body_version,
    //     //         ps.name,
    //     //         ps.category,
    //     //         ps.summary,
    //     //         ps.description,
    //     //         ps.timeline,
    //     //         ps.views,
    //     //         ps.submission_deadline
    //     //       FROM
    //     //         rfps p
    //     //         INNER JOIN (
    //     //           SELECT
    //     //             rfp_id,
    //     //             MAX(ts) AS max_ts
    //     //           FROM
    //     //             rfp_snapshots
    //     //           GROUP BY
    //     //             rfp_id
    //     //         ) latest_snapshots ON p.id = latest_snapshots.rfp_id
    //     //         INNER JOIN rfp_snapshots ps ON latest_snapshots.rfp_id = ps.rfp_id
    //     //         AND latest_snapshots.max_ts = ps.ts;
    //     //       "#
    //     //     )
    //     //     .fetch_all(&self.0)
    //     //     .await?;
    //     //     Ok(recs)
    //     // }

    //     // Additional functions can be added as needed
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match DB::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                rocket::error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(DB::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
    })
}

use rocket::{
    fairing::{self, AdHoc},
    Build, Rocket,
};
use rocket_db_pools::Database;
use sqlx::{
    migrate, query, query_as, query_file_as, query_scalar, types::BigDecimal, Error, PgPool,
    Postgres, Transaction,
};

#[derive(Database, Clone, Debug)]
#[database("devhub_cache")] // Adjust the database name accordingly
pub struct DB(PgPool);

pub mod types;

use devhub_shared::proposal::{Proposal, ProposalSnapshot};

use types::{
    AfterDate, ProposalRecord, ProposalSnapshotRecord, ProposalWithLatestSnapshotView,
    RfpSnapshotRecord,
};

// use crate::types::ProposalResponse;

impl DB {
    // Functions for Proposals
    pub async fn upsert_proposal(
        tx: &mut Transaction<'static, Postgres>,
        proposal_id: u32,
        author_id: String,
    ) -> Result<i32, Error> {
        let rec = sqlx::query!(
            r#"
            UPDATE proposals SET author_id = $1 WHERE id = $2
            RETURNING id
            "#,
            author_id,
            proposal_id as i32
        )
        .fetch_optional(tx.as_mut())
        .await?;

        // If the update did not find a matching row, insert the user
        if let Some(record) = rec {
            Ok(record.id)
        } else {
            // INSERT ON CONFLICT DO NOTHING
            let rec = sqlx::query!(
                r#"
                INSERT INTO proposals (id, author_id)
                VALUES ($1, $2)
                ON CONFLICT (id) DO NOTHING
                RETURNING id
                "#,
                proposal_id as i32,
                author_id
            )
            .fetch_one(tx.as_mut())
            .await?;
            Ok(rec.id)
        }
    }

    // TODO db.get_last_updated_timestamp
    pub async fn get_last_updated_timestamp(&self) -> Result<i64, Error> {
        // let rec = sqlx::query_file_as!(i64, "./sql/get_after_date.sql")
        //     .fetch_one(&self)
        //     .await?;

        let rec = query_scalar!(
            r#"
            SELECT after_date FROM after_date
            "#
        )
        .fetch_one(&self.0)
        .await?;
        Ok(rec)
    }

    pub async fn set_last_updated_timestamp(&self, after_date: i64) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE after_date SET after_date = $1
            "#,
            after_date
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }
    // TODO db.get_proposals
    pub async fn get_proposals(&self) -> Vec<ProposalRecord> {
        vec![]
    }

    pub async fn get_proposal_by_id(
        tx: &mut Transaction<'static, Postgres>,
        proposal_id: i32,
    ) -> anyhow::Result<Option<ProposalRecord>> {
        let rec = query!(
            r#"
          SELECT id, author_id
          FROM proposals
          WHERE id = $1
          "#,
            proposal_id
        )
        .fetch_optional(tx.as_mut())
        .await?;

        // Map the Record to Proposal
        let proposal = rec.map(|record| ProposalRecord {
            id: record.id,
            author_id: record.author_id,
            // social_db_post_block_height: 0,
            // snapshot: record.clone().snapshot,
            // snapshot_history: vec![],
            // Initialize other fields of Proposal if necessary
        });

        Ok(proposal)
    }

    pub async fn insert_proposal_snapshot(
        tx: &mut Transaction<'static, Postgres>,
        snapshot: &ProposalSnapshotRecord,
    ) -> anyhow::Result<()> {
        // Since primary key is (proposal_id, ts)
        query!(
            r#"
          INSERT INTO proposal_snapshots (
              proposal_id,
              block_height,
              ts,
              editor_id,
              social_db_post_block_height,
              labels,
              proposal_version,
              proposal_body_version,
              name,
              category,
              summary,
              description,
              linked_proposals,
              linked_rfp,
              requested_sponsorship_usd_amount,
              requested_sponsorship_paid_in_currency,
              requested_sponsor,
              receiver_account,
              supervisor,
              timeline,
              views
          ) VALUES (
              $1, $2, $3, $4, $5, $6, $7, $8,
              $9, $10, $11, $12, $13, $14,
              $15, $16, $17, $18, $19, $20, $21
          ) ON CONFLICT (proposal_id, ts) DO UPDATE SET
              block_height = $2,
              editor_id = $4,
              social_db_post_block_height = $5,
              labels = $6,
              proposal_version = $7,
              proposal_body_version = $8,
              name = $9,
              category = $10,
              summary = $11,
              description = $12,
              linked_proposals = $13,
              linked_rfp = $14,
              requested_sponsorship_usd_amount = $15,
              requested_sponsorship_paid_in_currency = $16,
              requested_sponsor = $17,
              receiver_account = $18,
              supervisor = $19,
              timeline = $20,
              views = $21
          "#,
            snapshot.proposal_id,
            snapshot.block_height,
            snapshot.ts,
            snapshot.editor_id,
            snapshot.social_db_post_block_height,
            snapshot.labels,
            snapshot.proposal_version,
            snapshot.proposal_body_version,
            snapshot.name,
            snapshot.category,
            snapshot.summary,
            snapshot.description,
            snapshot.linked_proposals,
            snapshot.linked_rfp,
            snapshot.requested_sponsorship_usd_amount,
            snapshot.requested_sponsorship_paid_in_currency,
            snapshot.requested_sponsor,
            snapshot.receiver_account,
            snapshot.supervisor,
            snapshot.timeline,
            snapshot.views
        )
        .execute(tx.as_mut())
        .await?;
        Ok(())
    }

    // pub async fn get_latest_proposal_snapshot(
    //     tx: &mut Transaction<'static, Postgres>,
    //     proposal_id: i32,
    // ) -> anyhow::Result<Option<ProposalSnapshot>> {
    //     let rec = query_as!(
    //         ProposalSnapshot,
    //         r#"
    //         SELECT * FROM proposal_snapshots
    //         WHERE proposal_id = $1
    //         ORDER BY ts DESC
    //         LIMIT 1
    //         "#,
    //         proposal_id
    //     )
    //     .fetch_optional(tx.as_mut())
    //     .await?;

    //     Ok(rec)
    // }

    // Functions for RFPs

    // pub async fn insert_rfp(
    //     tx: &mut Transaction<'static, Postgres>,
    //     author_id: &str,
    // ) -> anyhow::Result<i32> {
    //     let rec = sqlx::query!(
    //         r#"
    //       INSERT INTO rfps (author_id)
    //       VALUES ($1)
    //       RETURNING id
    //       "#,
    //         author_id
    //     )
    //     .fetch_one(tx)
    //     .await?;

    //     Ok(rec.id)
    // }

    // pub async fn upsert_rfp_snapshot(
    //     tx: &mut Transaction<'static, Postgres>,
    //     snapshot: &RfpSnapshot,
    // ) -> anyhow::Result<()> {
    // Primary key is (rfp_id, ts)
    //     sqlx::query!(
    //         r#"
    //       INSERT INTO rfp_snapshots (
    //           rfp_id,
    //           block_height,
    //           ts,
    //           editor_id,
    //           social_db_post_block_height,
    //           labels,
    //           linked_proposals,
    //           rfp_version,
    //           rfp_body_version,
    //           name,
    //           category,
    //           summary,
    //           description,
    //           timeline,
    //           submission_deadline,
    //           views
    //       ) VALUES (
    //           $1, $2, $3, $4, $5, $6, $7, $8,
    //           $9, $10, $11, $12, $13, $14, $15, $16
    //       ) ON CONFLICT (rfp_id, ts) DO UPDATE SET
    //           block_height = $2,
    //           editor_id = $4,
    //           social_db_post_block_height = $5,
    //           labels = $6,
    //           linked_proposals = $7,
    //           rfp_version = $8,
    //           rfp_body_version = $9,
    //           name = $10,
    //           category = $11,
    //           summary = $12,
    //           description = $13,
    //           timeline = $14,
    //           submission_deadline = $15,
    //           views = $16
    //       "#,
    //         snapshot.rfp_id,
    //         snapshot.block_height,
    //         snapshot.ts,
    //         snapshot.editor_id,
    //         snapshot.social_db_post_block_height,
    //         snapshot.labels,
    //         snapshot.linked_proposals,
    //         snapshot.rfp_version,
    //         snapshot.rfp_body_version,
    //         snapshot.name,
    //         snapshot.category,
    //         snapshot.summary,
    //         snapshot.description,
    //         snapshot.timeline,
    //         snapshot.submission_deadline,
    //         snapshot.views
    //     )
    //     .execute(tx)
    //     .await?;
    //     Ok(())
    // }

    // Function to get proposals with the latest snapshot

    pub async fn get_proposals_with_latest_snapshot(
        &self,
        limit: i64,
        order: &str,
    ) -> anyhow::Result<Vec<ProposalWithLatestSnapshotView>> {
        // Validate the order clause to prevent SQL injection
        let order_clause = match order.to_lowercase().as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => "DESC", // Default to DESC if the order is not recognized
        };

        // Build the SQL query with the validated order clause
        let sql = format!(
            r#"
            SELECT
                ps.proposal_id,
                p.author_id,
                ps.block_height,
                ps.ts,
                ps.editor_id,
                ps.social_db_post_block_height,
                ps.labels,
                ps.proposal_version,
                ps.proposal_body_version,
                ps.name,
                ps.category,
                ps.summary,
                ps.description,
                ps.linked_proposals,
                ps.linked_rfp,
                ps.requested_sponsorship_usd_amount::numeric::float8 AS "requested_sponsorship_usd_amount!",
                ps.requested_sponsorship_paid_in_currency,
                ps.requested_sponsor,
                ps.receiver_account,
                ps.supervisor,
                ps.timeline,
                ps.views
            FROM
                proposals p
            INNER JOIN (
                SELECT
                    proposal_id,
                    MAX(ts) AS max_ts
                FROM
                    proposal_snapshots
                GROUP BY
                    proposal_id
            ) latest_snapshots ON p.id = latest_snapshots.proposal_id
            INNER JOIN proposal_snapshots ps ON latest_snapshots.proposal_id = ps.proposal_id
                AND latest_snapshots.max_ts = ps.ts
            ORDER BY ps.ts {}
            LIMIT $1
            "#,
            order_clause,
        );

        // Execute the query
        let recs = sqlx::query_as::<_, ProposalWithLatestSnapshotView>(&sql)
            .bind(limit)
            .fetch_all(&self.0)
            .await?;

        Ok(recs)
    }

    // pub async fn get_proposals_with_latest_snapshot(
    //     &self,
    // ) -> anyhow::Result<Vec<ProposalWithLatestSnapshot>> {
    //     let recs = sqlx::query_as!(
    //         ProposalWithLatestSnapshot,
    //         r#"
    //       SELECT
    //         ps.proposal_id,
    //         p.author_id,
    //         ps.block_height,
    //         ps.ts,
    //         ps.editor_id,
    //         ps.social_db_post_block_height,
    //         ps.labels,
    //         ps.proposal_version,
    //         ps.proposal_body_version,
    //         ps.name,
    //         ps.category,
    //         ps.summary,
    //         ps.description,
    //         ps.linked_proposals,
    //         ps.linked_rfp,
    //         ps.requested_sponsorship_usd_amount,
    //         ps.requested_sponsorship_paid_in_currency,
    //         ps.requested_sponsor,
    //         ps.receiver_account,
    //         ps.supervisor,
    //         ps.timeline,
    //         ps.views
    //       FROM
    //         proposals p
    //         INNER JOIN (
    //           SELECT
    //             proposal_id,
    //             MAX(ts) AS max_ts
    //           FROM
    //             proposal_snapshots
    //           GROUP BY
    //             proposal_id
    //         ) latest_snapshots ON p.id = latest_snapshots.proposal_id
    //         INNER JOIN proposal_snapshots ps ON latest_snapshots.proposal_id = ps.proposal_id
    //         AND latest_snapshots.max_ts = ps.ts;
    //       "#
    //     )
    //     .fetch_all(&self.0)
    //     .await?;
    //     Ok(recs)
    // }

    // Function to get RFPs with the latest snapshot

    // pub async fn get_rfps_with_latest_snapshot(
    //     &self,
    // ) -> anyhow::Result<Vec<RfpWithLatestSnapshot>> {
    //     let recs = sqlx::query_as!(
    //         RfpWithLatestSnapshot,
    //         r#"
    //       SELECT
    //         ps.rfp_id,
    //         p.author_id,
    //         ps.block_height,
    //         ps.ts,
    //         ps.editor_id,
    //         ps.social_db_post_block_height,
    //         ps.labels,
    //         ps.linked_proposals,
    //         ps.rfp_version,
    //         ps.rfp_body_version,
    //         ps.name,
    //         ps.category,
    //         ps.summary,
    //         ps.description,
    //         ps.timeline,
    //         ps.views,
    //         ps.submission_deadline
    //       FROM
    //         rfps p
    //         INNER JOIN (
    //           SELECT
    //             rfp_id,
    //             MAX(ts) AS max_ts
    //           FROM
    //             rfp_snapshots
    //           GROUP BY
    //             rfp_id
    //         ) latest_snapshots ON p.id = latest_snapshots.rfp_id
    //         INNER JOIN rfp_snapshots ps ON latest_snapshots.rfp_id = ps.rfp_id
    //         AND latest_snapshots.max_ts = ps.ts;
    //       "#
    //     )
    //     .fetch_all(&self.0)
    //     .await?;
    //     Ok(recs)
    // }

    // Additional functions can be added as needed
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match DB::fetch(&rocket) {
        Some(db) => match migrate!("./migrations").run(&**db).await {
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

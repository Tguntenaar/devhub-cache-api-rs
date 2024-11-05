-- Add migration script here

-- blockheight became u64
-- timestamp became u64
-- requested_sponsorship_usd_amount became u32

DROP VIEW proposals_with_latest_snapshot;
DROP VIEW rfps_with_latest_snapshot;

ALTER TABLE proposal_snapshots
    ALTER COLUMN ts TYPE bigint USING ts::bigint,
    ALTER COLUMN requested_sponsorship_usd_amount TYPE integer USING requested_sponsorship_usd_amount::integer;


-- Migration script: Alter columns in dumps table
ALTER TABLE dumps
    ALTER COLUMN block_timestamp TYPE bigint USING block_timestamp::bigint,
    ALTER COLUMN proposal_id TYPE integer USING proposal_id::integer;

-- Migration script: Alter submission_deadline in rfp_snapshots table
ALTER TABLE rfp_snapshots
    ALTER COLUMN ts TYPE bigint USING ts::bigint,
    ALTER COLUMN submission_deadline TYPE bigint USING submission_deadline::bigint;

-- Migration script: Alter columns in rfp_dumps table
ALTER TABLE rfp_dumps
    ALTER COLUMN block_timestamp TYPE bigint USING block_timestamp::bigint,
    ALTER COLUMN rfp_id TYPE integer USING rfp_id::integer;

CREATE VIEW
  proposals_with_latest_snapshot AS
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
  ps.requested_sponsorship_usd_amount,
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
  AND latest_snapshots.max_ts = ps.ts;


CREATE VIEW
  rfps_with_latest_snapshot AS
SELECT
  ps.rfp_id,
  p.author_id,
  ps.block_height,
  ps.ts,
  ps.editor_id,
  ps.social_db_post_block_height,
  ps.labels,
  ps.linked_proposals,
  ps.rfp_version,
  ps.rfp_body_version,
  ps.name,
  ps.category,
  ps.summary,
  ps.description,
  ps.timeline,
  ps.views,
  ps.submission_deadline
FROM
  rfps p
  INNER JOIN (
    SELECT
      rfp_id,
      MAX(ts) AS max_ts
    FROM
      rfp_snapshots
    GROUP BY
      rfp_id
  ) latest_snapshots ON p.id = latest_snapshots.rfp_id
  INNER JOIN rfp_snapshots ps ON latest_snapshots.rfp_id = ps.rfp_id
  AND latest_snapshots.max_ts = ps.ts;

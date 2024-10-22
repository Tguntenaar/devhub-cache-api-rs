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
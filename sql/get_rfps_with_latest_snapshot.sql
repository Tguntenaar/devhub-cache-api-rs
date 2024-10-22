SELECT
    rs.rfp_id,
    r.author_id,
    rs.block_height,
    rs.ts,
    rs.editor_id,
    rs.social_db_post_block_height,
    rs.labels,
    rs.linked_proposals,
    rs.rfp_version,
    rs.rfp_body_version,
    rs.name,
    rs.category,
    rs.summary,
    rs.description,
    rs.timeline,
    rs.views,
    rs.submission_deadline
FROM
    rfps r
INNER JOIN (
    SELECT
        rfp_id,
        MAX(ts) AS max_ts
    FROM
        rfp_snapshots
    GROUP BY
        rfp_id
) latest_snapshots ON r.id = latest_snapshots.rfp_id
INNER JOIN rfp_snapshots rs ON latest_snapshots.rfp_id = rs.rfp_id
    AND latest_snapshots.max_ts = rs.ts;
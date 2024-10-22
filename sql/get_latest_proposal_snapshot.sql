SELECT
    ps.*
FROM
    proposal_snapshots ps
WHERE
    ps.proposal_id = $1
ORDER BY
    ps.ts DESC
LIMIT 1;


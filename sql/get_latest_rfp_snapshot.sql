SELECT
    rs.*
FROM
    rfp_snapshots rs
WHERE
    rs.rfp_id = $1
ORDER BY
    rs.ts DESC
LIMIT 1;
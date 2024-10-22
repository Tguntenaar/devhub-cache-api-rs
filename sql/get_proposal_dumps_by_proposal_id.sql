SELECT
    d.*
FROM
    dumps d
WHERE
    d.proposal_id = $1;
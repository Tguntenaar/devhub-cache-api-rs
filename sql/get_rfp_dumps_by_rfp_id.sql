SELECT
    rd.*
FROM
    rfp_dumps rd
WHERE
    rd.rfp_id = $1;
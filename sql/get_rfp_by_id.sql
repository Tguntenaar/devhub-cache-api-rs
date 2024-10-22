SELECT
    r.id,
    r.author_id
FROM
    rfps r
WHERE
    r.id = $1;
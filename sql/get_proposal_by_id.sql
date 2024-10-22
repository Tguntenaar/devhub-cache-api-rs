SELECT
    p.id,
    p.author_id
FROM
    proposals p
WHERE
    p.id = $1;
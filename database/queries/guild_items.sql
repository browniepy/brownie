SELECT
    id,
    name,
    description
FROM item
WHERE guild = $1;

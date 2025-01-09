SELECT
    item.id,
    item.name,
    shop.price,
    shop.description
FROM
    shop
    JOIN item ON shop.item = item.id
WHERE
    shop.guild = $1;

CREATE TABLE gamble_log (
    id serial PRIMARY KEY,
    name varchar(255) NOT NULL,
    rpg boolean NOT NULL DEFAULT FALSE,
    solo boolean NOT NULL DEFAULT FALSE,
    first_member bigint NOT NULL,
    second_member bigint,
    winner bigint,
    loser bigint,
    balance bigint NOT NULL,
    created_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    CONSTRAINT solo_check CHECK ((solo = TRUE AND second_member IS NULL) OR (solo = FALSE AND second_member IS NOT NULL))
);

CREATE TABLE gamble_items_log (
    gamble_log int REFERENCES gamble_log (id) ON DELETE CASCADE,
    item int NOT NULL,
    amount int NOT NULL,
    PRIMARY KEY (gamble_log, item)
);

CREATE FUNCTION log_gamble_item (gamble_log int, item int, amount int)
    RETURNS VOID
    AS $$
BEGIN
    INSERT INTO gamble_items_log (gamble_log, item, amount)
        VALUES (gamble_log, item, amount)
    ON CONFLICT (gamble_log, item)
        DO UPDATE SET
            amount = gamble_items_log.amount + amount;
END;
$$
LANGUAGE plpgsql;


CREATE TYPE ROLE AS ENUM (
    'Player',
    'Gambler'
);

CREATE TABLE IF NOT EXISTS member (
    id bigint PRIMARY KEY,
    name varchar(32),
    roles ROLE[] NOT NULL DEFAULT ARRAY['Player'] ::role[],
    registered_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    balance bigint NOT NULL DEFAULT 1000,
    points int NOT NULL DEFAULT 0
);

INSERT INTO member (id)
VALUES (1323132058184192030);

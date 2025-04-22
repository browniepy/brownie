CREATE TABLE clandestine (
    id serial PRIMARY KEY,
    start_date timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    end_date timestamp
);

CREATE TABLE ilegal_participants (
    season int REFERENCES clandestine (id) ON DELETE CASCADE,
    participant bigint REFERENCES member (id) ON DELETE CASCADE,
    UNIQUE (season, participant)
);

CREATE TABLE clandestine_fight (
    id serial PRIMARY KEY,
    season int REFERENCES clandestine (id) ON DELETE CASCADE,
    winner bigint REFERENCES member (id) ON DELETE CASCADE
);


CREATE TABLE normal_stats (
    id bigint REFERENCES member (id) ON DELETE CASCADE,
    strength int DEFAULT 1,
    endurance int DEFAULT 1,
    violence int DEFAULT 1,
    stamina int DEFAULT 1,
    reaction int DEFAULT 1,
    precision int DEFAULT 1,
    clandestine_wins int DEFAULT 0,
    kills int DEFAULT 0,
    fights int DEFAULT 0
);


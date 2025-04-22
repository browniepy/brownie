CREATE TYPE rpg_state AS ENUM (
    'Active',
    'Nightmare',
    'End'
);

CREATE TYPE rpg_role AS ENUM (
    'Knight',
    'King',
    'Coordinator'
);

CREATE TABLE rpg (
    id serial PRIMARY KEY,
    started_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    ended_at timestamp,
    state RPG_STATE NOT NULL DEFAULT 'Active' ::rpg_state
);

CREATE TYPE economic_class AS ENUM (
    'Slave',
    'Citizien',
    'Normal'
);

CREATE TABLE player (
    rpg int REFERENCES rpg (id) ON DELETE CASCADE,
    player bigint REFERENCES member (id) ON DELETE CASCADE,
    start_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    end_at timestamp,
    playing boolean DEFAULT TRUE,
    outlaw boolean DEFAULT FALSE,
    balance bigint NOT NULL DEFAULT 10,
    level INT NOT NULL DEFAULT 1,
    experience int NOT NULL DEFAULT 0,
    role RPG_ROLE[] NOT NULL DEFAULT ARRAY[] ::rpg_role[],
    PRIMARY KEY (rpg, player)
);

CREATE TYPE rpg_skill_type AS ENUM (
    'Combat',
    'Loot',
    'Life'
);

CREATE TABLE rpg_skill (
    id serial PRIMARY KEY,
    skill_type RPG_SKILL_TYPE NOT NULL,
    max_level int NOT NULL DEFAULT 100
);

CREATE TYPE skill_stat_type AS ENUM (
    'Accuracy',
    'Damage',
    'MaxLife',
    'Defense',
    'Quantity',
    'Quality'
);

CREATE TABLE player_skill (
    rpg int NOT NULL,
    player bigint NOT NULL,
    skill int NOT NULL REFERENCES rpg_skill (id) ON DELETE CASCADE,
    level INT NOT NULL DEFAULT 1,
    experience int NOT NULL DEFAULT 0,
    available_points int NOT NULL DEFAULT 0,
    PRIMARY KEY (rpg, player, skill),
    FOREIGN KEY (rpg, player) REFERENCES player (rpg, player) ON DELETE CASCADE
);

CREATE TABLE stat_points (
    rpg int NOT NULL,
    player bigint NOT NULL,
    skill int NOT NULL,
    stat SKILL_STAT_TYPE NOT NULL,
    points int NOT NULL DEFAULT 0,
    PRIMARY KEY (rpg, player, skill, stat),
    FOREIGN KEY (rpg, player, skill) REFERENCES player_skill (rpg, player, skill) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_rpg_only_one_active ON rpg ((1))
WHERE
    state = 'Active';


CREATE TABLE rpg_quest (
    id serial PRIMARY KEY,
    name varchar(50) NOT NULL,
    quest_type QUEST_TYPE NOT NULL,
    item int REFERENCES rpg_item (id) ON DELETE SET NULL,
    amount int DEFAULT 0,
    experience int DEFAULT 0,
    bios int DEFAULT 0
);

CREATE TABLE player_quests (
    rpg int NOT NULL,
    player bigint NOT NULL,
    quest int NOT NULL REFERENCES rpg_quest (id) ON DELETE CASCADE,
    started_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    ended_at timestamp,
    amount int DEFAULT 0,
    PRIMARY KEY (rpg, player, quest),
    FOREIGN KEY (rpg, player) REFERENCES player (rpg, player) ON DELETE CASCADE
);


CREATE TABLE player_equipment (
    rpg int NOT NULL,
    player bigint NOT NULL,
    head int REFERENCES rpg_item (id) ON DELETE SET NULL,
    chest int REFERENCES rpg_item (id) ON DELETE SET NULL,
    legs int REFERENCES rpg_item (id) ON DELETE SET NULL,
    boots int REFERENCES rpg_item (id) ON DELETE SET NULL,
    first_ring int REFERENCES rpg_item (id) ON DELETE SET NULL,
    second_ring int REFERENCES rpg_item (id) ON DELETE SET NULL,
    neck int REFERENCES rpg_item (id) ON DELETE SET NULL,
    first_hand int REFERENCES rpg_item (id) ON DELETE SET NULL,
    second_hand int REFERENCES rpg_item (id) ON DELETE SET NULL,
    PRIMARY KEY (rpg, player),
    FOREIGN KEY (rpg, player) REFERENCES player (rpg, player) ON DELETE CASCADE
);


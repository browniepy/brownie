CREATE TABLE rpg_item (
    id serial PRIMARY KEY,
    name varchar(50) NOT NULL,
    usable boolean NOT NULL DEFAULT FALSE,
    tool_type TOOL_TYPE,
    item_type ITEM_TYPE NOT NULL,
    armor_type ARMOR_TYPE,
    two_handed boolean NOT NULL DEFAULT FALSE,
    quality QUALITY NOT NULL DEFAULT 'Common' ::quality
);

CREATE TABLE rpg_craft (
    craft_item int NOT NULL REFERENCES rpg_item (id) ON DELETE CASCADE,
    recipe_item int NOT NULL REFERENCES rpg_item (id) ON DELETE CASCADE,
    amount int NOT NULL,
    PRIMARY KEY (craft_item, recipe_item)
);

CREATE TABLE player_inventory (
    rpg int NOT NULL,
    player bigint NOT NULL,
    item int NOT NULL REFERENCES rpg_item (id) ON DELETE CASCADE,
    amount int NOT NULL DEFAULT 0,
    PRIMARY KEY (rpg, player, item),
    FOREIGN KEY (rpg, player) REFERENCES player (rpg, player) ON DELETE CASCADE
);

CREATE TABLE rpg_shop (
    item int NOT NULL REFERENCES rpg_item (id) ON DELETE CASCADE,
    price int,
    PRIMARY KEY (item)
);


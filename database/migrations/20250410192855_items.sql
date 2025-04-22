CREATE TYPE item_type AS ENUM (
    'Equipment',
    'Tool',
    'Material',
    'Quest',
    'Misc',
    'Consumable'
);

CREATE TYPE tool_type AS ENUM (
    'Weapon',
    'Shield',
    'Accesory',
    'Pickaxe',
    'Axe'
);

CREATE TYPE quality AS ENUM (
    'Common',
    'Normal',
    'Epic',
    'Masterpiece'
);

CREATE TYPE armor_type AS ENUM (
    'Head',
    'Chest',
    'Legs',
    'Boots',
    'Neck',
    'Ring'
);

CREATE TABLE IF NOT EXISTS normal_item (
    id serial PRIMARY KEY,
    name varchar(50) NOT NULL,
    number int,
    usable boolean NOT NULL DEFAULT FALSE,
    item_type item_type NOT NULL,
    quality quality NOT NULL DEFAULT 'Common',
    victim bigint
);

CREATE TABLE IF NOT EXISTS normal_inventory (
    item int REFERENCES normal_item (id) ON DELETE CASCADE,
    member bigint REFERENCES member (id) ON DELETE CASCADE,
    amount int NOT NULL DEFAULT 0,
    PRIMARY KEY (item, member)
);

CREATE TABLE IF NOT EXISTS normal_shop (
    item int REFERENCES normal_item (id) ON DELETE CASCADE,
    stock int,
    price int,
    PRIMARY KEY (item)
);


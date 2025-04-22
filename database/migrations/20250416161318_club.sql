CREATE TABLE club (
    id bigserial PRIMARY KEY,
    leader bigint REFERENCES member (id) ON DELETE SET NULL,
    created_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    name varchar(255) NOT NULL,
    description varchar(1500),
    prestige int DEFAULT 0,
    points int DEFAULT 0,
    UNIQUE (name)
);

CREATE TYPE authority_id AS ENUM (
    'Leader',
    'Member',
    'Agent'
);

CREATE TABLE translations_keys (
  trans_key varchar(50) PRIMARY KEY
);

CREATE TABLE translations (
  trans_key varchar(50) REFERENCES translations_keys(trans_key) ON DELETE CASCADE ON UPDATE CASCADE,
  locale varchar(10) NOT NULL,
  trans_value VARCHAR(500) NOT NULL,
  PRIMARY KEY (trans_key, locale)
);

CREATE TABLE club_role (
    club bigint REFERENCES club (id) ON DELETE CASCADE,
    authority int NOT NULL,
    authority_id authority_id,

    tr_key varchar(50) NOT NULL,
    PRIMARY KEY (club, tr_key)
);

CREATE TYPE club_item_type AS ENUM (
    'Membership',
    'Agent'
);

CREATE TABLE club_role_item (
    club bigint,
    role_tr_key varchar(50),
    item_tr_key varchar(50) NOT NULL,
    item_type club_item_type NOT NULL,
    PRIMARY KEY (club, role_tr_key),
    FOREIGN KEY (club, role_tr_key) REFERENCES club_role (club, tr_key)
);

CREATE TABLE club_member (
    club bigint,
    member bigint REFERENCES member (id) ON DELETE CASCADE,
    agent_range int,
    role_name varchar(255) NOT NULL,
    PRIMARY KEY (club, member),
    FOREIGN KEY (club, role_name) REFERENCES club_role (club, tr_key) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE club_limits (
    club bigint,
    role_name varchar(255) NOT NULL,
    member_limit int NOT NULL CHECK (member_limit <= 150),
    PRIMARY KEY (club, role_name),
    FOREIGN KEY (club, role_name) REFERENCES club_role (club, tr_key) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE FUNCTION check_club_member_role_limit ()
    RETURNS TRIGGER
    AS $$
DECLARE
    member_count int;
    role_member_limit int;
BEGIN
    SELECT
        cl.member_limit INTO role_member_limit
    FROM
        club_limits cl
    WHERE
        cl.club = NEW.club
        AND cl.role_name = NEW.role_name;
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;
    SELECT
        COUNT(*) INTO member_count
    FROM
        club_member cm
    WHERE
        cm.club = NEW.club
        AND cm.role_name = NEW.role_name;
    IF member_count >= role_member_limit THEN
        RETURN NULL;
    END IF;
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER check_club_member_role_limit
    BEFORE INSERT OR UPDATE ON club_member
    EXECUTE FUNCTION check_club_member_role_limit ();


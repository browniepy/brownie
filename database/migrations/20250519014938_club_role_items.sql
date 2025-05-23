CREATE TYPE club_role_item_type AS ENUM (
  'Membership',
  'Agent'
);

CREATE TABLE club_role_item (
  club BIGINT NOT NULL,
  role_tr_key VARCHAR(255) NOT NULL,
  item_tr_key VARCHAR(255) NOT NULL,
  item_type club_role_item_type NOT NULL,
  FOREIGN KEY (club, role_tr_key) REFERENCES club_role (club, tr_key) ON DELETE CASCADE,
  PRIMARY KEY (club, role_tr_key)
);

ALTER TABLE club_role DROP COLUMN item_tr_key;

CREATE OR REPLACE FUNCTION create_club_item(
  club_id BIGINT,
  role_name VARCHAR(255),
  item_name VARCHAR(255),
  item_type club_role_item_type
)
RETURNS BOOLEAN AS $$
DECLARE
  item_exists BOOLEAN;
BEGIN
  SELECT id
  FROM club
  WHERE id = club_id;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'club not found';
  END IF;

  SELECT EXISTS (
    SELECT item_tr_key
    FROM club_role_item
    WHERE club = club_id
    AND role_tr_key = role_name
  ) INTO item_exists;

  IF item_exists THEN
    RAISE EXCEPTION 'role item already exists';
  END IF;

  INSERT INTO club_role_item (
    club, role_tr_key, item_tr_key, item_type
  ) VALUES ( club_id, role_name, item_name, item_type );

  RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

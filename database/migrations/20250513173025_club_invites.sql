CREATE TABLE club_apply (
  id SERIAL,
  club BIGINT REFERENCES club(id) ON DELETE CASCADE,
  member BIGINT REFERENCES member(id) ON DELETE CASCADE,
  send_at TIMESTAMP NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
  completed BOOL NOT NULL,
  PRIMARY KEY (club, member)
);

CREATE OR REPLACE FUNCTION club_send_apply(
  club_id BIGINT,
  member_id BIGINT
)
RETURNS BOOLEAN AS $$
DECLARE
  member_complete_req BOOLEAN := TRUE;
  member_balance BIGINT;
  member_points INT;
  req_balance INT;
  req_points INT;
BEGIN
  SELECT id
  FROM club
  WHERE id = club_id;

  IF NOT FOUND THEN
    raise 'club not found';
  END IF;

  SELECT id
  FROM club_apply
  WHERE club = club_id
  AND member = member_id;

  IF FOUND THEN
    raise 'member already send apply to this club';
  END IF;

  SELECT balance, points
  FROM club_req
  WHERE club = club_id
  INTO req_balance, req_points;

  SELECT balance, points
  FROM member
  WHERE id = member_id
  INTO member_balance, member_points;

  IF member_balance != req_balance THEN
    member_complete_req = FALSE;
  END IF;

  IF member_points != req_points THEN
    member_complete_req = FALSE;
  END IF;

  INSERT INTO club_apply (club, member, completed)
  VALUES (club_id, member_id, member_complete_req);

  RETURN member_complete_req;
END;
$$ LANGUAGE plpgsql;

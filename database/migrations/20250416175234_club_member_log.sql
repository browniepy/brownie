CREATE TABLE club_role_log (
    club bigint REFERENCES club (id) ON DELETE CASCADE,
    member bigint REFERENCES member (id) ON DELETE CASCADE,
    assigned_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    role_name varchar(255) NOT NULL,
    nth int NOT NULL
);

CREATE FUNCTION log_club_role_change ()
    RETURNS TRIGGER
    AS $$
DECLARE
    next_nth int;
BEGIN
    IF (TG_OP = 'INSERT' OR (TG_OP = 'UPDATE' AND NEW.role_name IS DISTINCT FROM OLD.role_name)) THEN
        SELECT
            COALESCE(MAX(nth), 0) + 1 INTO next_nth
        FROM
            club_role_log
        WHERE
            club = NEW.club
            AND role_name = NEW.role_name;
        INSERT INTO club_role_log (club, member, role_name, nth)
            VALUES (NEW.club, NEW.member, NEW.role_name, next_nth);
    END IF;
    RETURN NEW;
END
$$
LANGUAGE plpgsql;

CREATE TRIGGER log_club_role_change
    BEFORE INSERT OR UPDATE ON club_member
    EXECUTE FUNCTION log_club_role_change ();


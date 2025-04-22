CREATE TABLE IF NOT EXISTS job (
    id serial PRIMARY KEY,
    name varchar(50) NOT NULL,
    salary int[] NOT NULL,
    required_points int NOT NULL DEFAULT 0,
    required_role ROLE,
    cooldown int NOT NULL DEFAULT 300
);

ALTER TABLE member
    ADD COLUMN job INT REFERENCES job (id) ON DELETE SET NULL;

CREATE OR REPLACE FUNCTION can_apply_job (member bigint, job int)
    RETURNS boolean
    AS $$
DECLARE
    member_points int;
    member_roles ROLE[];
    required_points int;
    required_role ROLE;
BEGIN
    SELECT
        points,
        roles INTO member_points,
        member_roles
    FROM
        member
    WHERE
        id = member;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'member not found';
    END IF;
    SELECT
        required_points,
        required_role INTO required_points,
        required_role
    FROM
        job
    WHERE
        id = job;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'job not found';
    END IF;
    IF member_points < required_points THEN
        RETURN FALSE;
    END IF;
    IF required_role IS NOT NULL AND NOT (required_role = ANY (member_roles)) THEN
        RETURN FALSE;
    END IF;
END;
$$
LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION apply_job (member bigint, job int)
    RETURNS boolean
    AS $$
DECLARE
    can_apply boolean;
BEGIN
    SELECT
        can_apply_job (member, job) INTO can_apply;
    IF NOT can_apply THEN
        RETURN FALSE;
    END IF;
    UPDATE
        member
    SET
        job = job
    WHERE
        id = member;
END;
$$
LANGUAGE plpgsql;


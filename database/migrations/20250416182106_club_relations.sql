CREATE TABLE agent_slots (
    club bigint REFERENCES club (id) ON DELETE CASCADE,
    agent_range int PRIMARY KEY,
    occupied boolean DEFAULT FALSE
);

CREATE FUNCTION create_club (leader bigint, club_name varchar(255), leader_role_name varchar(255), agent_role_name varchar(255), member_role_name varchar(255))
    RETURNS VOID
    AS $$
DECLARE
    club_id bigint;
    agent_limit int;
BEGIN
    INSERT INTO club (leader, name)
        VALUES (leader, club_name)
    RETURNING
        id INTO club_id;

    INSERT INTO club_role (club, tr_key, authority, authority_id)
        VALUES (club_id, leader_role_name, 100, 'Leader');

    INSERT INTO club_limits (club, role_name, member_limit)
        VALUES (club_id, leader_role_name, 1);

    IF leader IS NOT NULL THEN
        INSERT INTO club_member (club, member, tr_key)
            VALUES (club_id, leader, leader_role_name);
    END IF;

    INSERT INTO club_role (club, tr_key, authority, authority_id)
        VALUES (club_id, member_role_name, 10, 'Member');
    INSERT INTO club_limits (club, role_name, member_limit)
        VALUES (club_id, member_role_name, 48);

    INSERT INTO club_role (club, tr_key, authority, authority_id)
        VALUES (club_id, agent_role_name, 70, 'Agent');
    INSERT INTO club_limits (club, role_name, member_limit)
        VALUES (club_id, agent_role_name, 101)
    RETURNING
        member_limit INTO agent_limit;

    INSERT INTO agent_slots (club, agent_range)
    SELECT
        club_id,
        generate_series(0, agent_limit);
END;
$$
LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS agent_relation (
    club bigint,
    member bigint,
    agent bigint,
    created_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    FOREIGN KEY (club, member) REFERENCES club_member (club, member) ON DELETE CASCADE,
    PRIMARY KEY (club, member, agent)
);

SELECT
    create_club (NULL, 'Kakerou', 'leader', 'referee', 'member');

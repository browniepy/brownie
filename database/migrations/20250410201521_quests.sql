CREATE TYPE quest_type AS ENUM (
    'Game',
    'Work',
    'Bet',
    'Kill',
    'Dungeon',
    'Exploration',
    'Eat',
    'Visit'
);

CREATE TABLE IF NOT EXISTS random_quest (
    id serial PRIMARY KEY,
    name varchar(50) NOT NULL,
    required_steps int NOT NULL,
    game_name varchar(50),
    points int DEFAULT 0,
    yn int DEFAULT 0
);

CREATE TABLE member_random_quests (
    id serial PRIMARY KEY,
    member bigint REFERENCES member (id) ON DELETE CASCADE,
    quest int REFERENCES random_quest (id) ON DELETE CASCADE,
    assigned_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    completed_at timestamp,
    UNIQUE (member, quest, assigned_at)
);

CREATE FUNCTION generate_random_quests (member_id bigint)
    RETURNS VOID
    AS $$
DECLARE
    CURRENT_DATE DATE := CURRENT_DATE;
    quests_today int;
BEGIN
    SELECT
        COUNT(*) INTO quests_today
    FROM
        member_random_quests
    WHERE
        member = member_id
        AND completed_at IS NULL
        AND DATE(assigned_at) = CURRENT_DATE;
    IF quests_today = 0 THEN
        DELETE FROM member_random_quests
        WHERE member = member_id
            AND completed_at IS NULL
            AND DATE(assigned_at) < CURRENT_DATE;
        INSERT INTO member_random_quests (member, quest)
        SELECT
            member_id,
            id
        FROM
            random_quest
        ORDER BY
            RANDOM()
        LIMIT 3;
    END IF;
END;
$$
LANGUAGE plpgsql;

CREATE FUNCTION random_quest_streak (member_id bigint)
    RETURNS int
    AS $$
DECLARE
    streak int := 0;
    actual_date date := CURRENT_DATE;
    day_to_check date := actual_date;
    quests_completed boolean;
BEGIN
    FOR i IN 1..7 LOOP
        day_to_check := CURRENT_DATE - i * INTERVAL '1 day';
        SELECT
            EXISTS (
                SELECT
                    1
                FROM
                    member_random_quests
                WHERE
                    member = member_id
                    AND COMPLETEd_at IS NOT NULL
                    AND DATE(assigned_at) = day_to_check) INTO quests_completed;
        IF NOT quests_completed THEN
            EXIT;
        END IF;
        streak := streak + 1;
    END LOOP;
    RETURN streak;
END;
$$
LANGUAGE plpgsql;


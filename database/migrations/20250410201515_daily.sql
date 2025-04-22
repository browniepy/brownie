CREATE TABLE IF NOT EXISTS daily_reward (
    member bigint REFERENCES member (id) ON DELETE CASCADE,
    claimed_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    UNIQUE (member, claimed_at)
);

CREATE FUNCTION can_claim_daily_reward (member_id bigint)
    RETURNS boolean
    LANGUAGE plpgsql
    AS $$
DECLARE
    last_claim_date date;
BEGIN
    SELECT
        DATE(claimed_at) INTO last_claim_date
    FROM
        daily_reward
    WHERE
        member = member_id
    ORDER BY
        claimed_at DESC
    LIMIT 1;
    RETURN last_claim_date IS NULL
        OR last_claim_date < CURRENT_DATE;
END;
$$;

CREATE FUNCTION register_daily_reward_claim (member_id bigint)
    RETURNS boolean
    LANGUAGE plpgsql
    AS $$
DECLARE
    last_claim_date date;
    current_utc_timestamp timestamp with time zone;
BEGIN
    current_utc_timestamp := NOW() AT TIME ZONE 'UTC';
    SELECT
        (claimed_at AT TIME ZONE 'UTC')::date INTO last_claim_date
    FROM
        daily_reward
    WHERE
        member = member_id
    ORDER BY
        claimed_at DESC
    LIMIT 1;
    IF last_claim_date IS NULL OR last_claim_date < (current_utc_timestamp)::date THEN
        INSERT INTO daily_reward (member, claimed_at)
            VALUES (member_id, current_utc_timestamp);
        RETURN TRUE;
    ELSE
        RETURN FALSE;
    END IF;
END;
$$;

CREATE FUNCTION daily_streak (member_id bigint)
    RETURNS int
    AS $$
DECLARE
    streak int := 0;
    actual_date date := CURRENT_DATE;
    day_to_check date := actual_date;
    reward_claimed boolean;
BEGIN
    FOR i IN 1..7 LOOP
        day_to_check := CURRENT_DATE - i * INTERVAL '1 day';
        SELECT
            EXISTS (
                SELECT
                    1
                FROM
                    daily_reward
                WHERE
                    member = member_id
                    AND DATE(claimed_at) = day_to_check) INTO reward_claimed;
        IF NOT reward_claimed THEN
            EXIT;
        END IF;
        streak := streak + 1;
    END LOOP;
    RETURN streak;
END;
$$
LANGUAGE plpgsql;


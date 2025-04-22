CREATE TABLE command_log (
    member bigint NOT NULL REFERENCES member (id) ON DELETE CASCADE,
    used_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    name varchar(255) NOT NULL,
    amount bigint NOT NULL,
    PRIMARY KEY (member, name)
);

CREATE FUNCTION log_command (member bigint, name varchar(255))
    RETURNS VOID
    AS $$
BEGIN
    INSERT INTO command_log (member, name, amount)
        VALUES (member, name, 1)
    ON CONFLICT (member, name)
        DO UPDATE SET
            amount = command_log.amount + 1;
END;
$$
LANGUAGE plpgsql;


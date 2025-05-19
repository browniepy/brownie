CREATE TYPE club_type AS ENUM (
    'Club',
    'Academy',
    'Organization',
    'Mafia',
    'Fundation',
    'Group'
);

CREATE TABLE club (
    id bigserial PRIMARY KEY,
    leader bigint REFERENCES member (id) ON DELETE SET NULL,
    created_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    name varchar(255) NOT NULL,
    description varchar(1500),
    prestige int NOT NULL DEFAULT 0,
    points int NOT NULL DEFAULT 0,
    renameable boolean NOT NULL DEFAULT true,
    deleteable boolean NOT NULL DEFAULT true,
    bank BIGINT NOT NULL DEFAULT 0,
    total_invested BIGINT NOT NULL DEFAULT 0,
    club_type club_type NOT NULL DEFAULT 'Club',
    applications_enabled boolean NOT NULL DEFAULT false,
    UNIQUE (name)
);

CREATE TABLE club_join_requirements (
  club bigint REFERENCES club (id) ON DELETE CASCADE,
  required_balance bigint DEFAULT 0,
  required_level int DEFAULT 0
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

CREATE TYPE perm AS ENUM (
    'ManageRoles',
    'ManageMembers',
    'ManageBank',
    'ManageClub',
    'InviteMembers',
    'All'
);

CREATE TABLE club_role (
    club bigint REFERENCES club (id) ON DELETE CASCADE,
    authority int NOT NULL,
    authority_id authority_id,
    perms perm[] NOT NULL DEFAULT ARRAY[]::perm[],
    item_tr_key varchar(50),

    tr_key varchar(50) NOT NULL,
    PRIMARY KEY (club, tr_key)
);

CREATE TYPE club_item_type AS ENUM (
    'Membership',
    'Agent'
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

CREATE TABLE club_stl_rules (
  club bigint,
  required_role varchar(255),
  required_balance bigint NOT NULL DEFAULT 50000000,
  karamete BOOLEAN NOT NULL DEFAULT false,
  required_agent_zero BOOLEAN NOT NULL DEFAULT false,
  FOREIGN KEY (club, required_role) REFERENCES club_role (club, tr_key) ON DELETE CASCADE ON UPDATE CASCADE
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

-- Tabla para almacenar los logs de cambios de roles
CREATE TABLE IF NOT EXISTS club_role_log (
    id BIGSERIAL PRIMARY KEY,
    club BIGINT REFERENCES club (id) ON DELETE CASCADE,
    member BIGINT REFERENCES member (id) ON DELETE CASCADE,
    assigned_at TIMESTAMP NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    role_name VARCHAR(255) NOT NULL,
    nth INT NOT NULL,
    assigned_by BIGINT REFERENCES member (id) ON DELETE SET NULL,  -- Puede ser NULL para asignaciones del sistema
    previous_role VARCHAR(255),
    system_assigned BOOLEAN DEFAULT FALSE  -- Flag para indicar si fue asignado por el sistema
);

CREATE OR REPLACE FUNCTION log_club_role(
    p_club_id BIGINT,
    p_member_id BIGINT,
    p_role_name VARCHAR(255),
    p_assigned_by BIGINT DEFAULT NULL,  -- Parámetro opcional
    p_previous_role VARCHAR(255) DEFAULT NULL
)
RETURNS VOID AS $$
DECLARE
    next_nth INT;
BEGIN
    -- Calcular el siguiente número de secuencia para este rol en el club
    SELECT 
        COALESCE(MAX(nth), 0) + 1 INTO next_nth
    FROM 
        club_role_log
    WHERE 
        club = p_club_id
        AND role_name = p_role_name;

    -- Insertar el registro en la tabla de logs
    INSERT INTO club_role_log (
        club, 
        member, 
        role_name, 
        nth, 
        assigned_by, 
        previous_role,
        system_assigned
    )
    VALUES (
        p_club_id, 
        p_member_id, 
        p_role_name, 
        next_nth, 
        p_assigned_by, 
        p_previous_role,
        p_assigned_by IS NULL
    );
END;
$$ LANGUAGE plpgsql;

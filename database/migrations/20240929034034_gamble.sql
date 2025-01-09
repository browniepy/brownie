-- Add migration script here

CREATE TYPE roles_enum AS ENUM ('Baku', 'Slave', 'User', 'Member', 'Leader', 'Referee', 'Perfect', 'Gambler');

CREATE TABLE job (
    name VARCHAR(255) PRIMARY KEY,
    description VARCHAR(255) DEFAULT NULL,
    salary_range INT[] NOT NULL,
    required_level INT DEFAULT 1 NOT NULL,
    required_role roles_enum
);

INSERT INTO job (name, salary_range, required_role)
VALUES ('Referee', '{4200, 6000}', 'Referee');

CREATE TABLE member (
  id BIGINT PRIMARY KEY,
  roles roles_enum[],

  job VARCHAR(255) REFERENCES job(name) DEFAULT NULL,

  balance INT DEFAULT 1000 NOT NULL,
  points INT DEFAULT 0 NOT NULL,
  level INT DEFAULT 1 NOT NULL,

  reputation INT DEFAULT 0 NOT NULL,

  referee_range INT DEFAULT NULL,
  personal_referee BIGINT REFERENCES member(id) DEFAULT NULL,
  profile_text VARCHAR(255) DEFAULT NULL,

  CONSTRAINT check_personal CHECK (
    ('Referee' = ANY(roles) AND personal_referee IS NULL) OR
    NOT 'Referee' = ANY(roles)
  ),

  CONSTRAINT check_range CHECK (
    ('Referee' = ANY(roles) AND referee_range BETWEEN 0 AND 100) OR
    NOT 'Referee' = ANY(roles)
  )
);

-- table to count victories and defeats in a game
CREATE TABLE statistics (
    member BIGINT REFERENCES member(id),
    game TEXT NOT NULL,
    victories INT DEFAULT 0 NOT NULL,
    defeats INT DEFAULT 0 NOT NULL,
    victory_text VARCHAR(255) DEFAULT NULL,
    defeat_text VARCHAR(255) DEFAULT NULL,
    PRIMARY KEY (member, game)
);

CREATE TABLE referee_slots (
  referee_range INT PRIMARY KEY,
  occupied BOOLEAN DEFAULT FALSE
);

INSERT INTO referee_slots (referee_range)
SELECT generate_series(0, 100);

CREATE OR REPLACE FUNCTION assign_referee_range(member_id BIGINT)
RETURNS VOID AS $$
DECLARE
  available_range INT;
  current_roles roles_enum[];
BEGIN
  -- Obtener los roles actuales del miembro
  SELECT roles INTO current_roles FROM member WHERE id = member_id;

  -- Verificar si el miembro ya es Referee. Si lo es, no hacer nada.
    IF 'Referee' = ANY(current_roles) THEN
        RETURN;
    END IF;


  -- Verificar las restricciones de roles
  IF 'Member' = ANY(current_roles) OR 'Leader' = ANY(current_roles) THEN
    RAISE EXCEPTION 'Referee cannot be assigned to Members or Leaders';
  END IF;


  -- Encontrar un rango disponible
  SELECT referee_range INTO available_range
  FROM referee_slots
  WHERE occupied = FALSE
  LIMIT 1;

  -- Si no hay rangos disponibles, lanzar una excepción.
  IF NOT FOUND THEN
    RAISE EXCEPTION 'No hay rangos de referí disponibles.';
  END IF;

  -- Actualizar el miembro y el slot de referí
  UPDATE member SET referee_range = available_range, roles = array_append(current_roles, 'Referee')
  WHERE id = member_id;

  UPDATE referee_slots SET occupied = TRUE
  WHERE referee_range = available_range;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION release_referee_range(member_id BIGINT)
RETURNS VOID AS $$
DECLARE
    referee_range_to_release INT;
BEGIN
    SELECT referee_range INTO referee_range_to_release FROM member WHERE id = member_id;

    -- Actualizar el miembro y el slot de referí, quitando 'Referee' del array
  UPDATE member SET referee_range = NULL, roles = array_remove(roles, 'Referee')
  WHERE id = member_id;

  UPDATE referee_slots SET occupied = FALSE
  WHERE referee_range = referee_range_to_release;

END;
$$ LANGUAGE plpgsql;

CREATE TABLE item (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT
);

CREATE TABLE inventory (
  item INT REFERENCES item(id),
  member BIGINT REFERENCES member(id),
  PRIMARY KEY (item, member),
  amount INT
);

CREATE TABLE store (
  item INT REFERENCES item(id),
  PRIMARY KEY(item),
  price INT,
  description TEXT
);

CREATE OR REPLACE FUNCTION check_member_limit()
RETURNS TRIGGER AS $$
DECLARE
  member_count INT;
BEGIN
  -- Contar el número de miembros con rol 'Member' en el array de roles
  SELECT COUNT(*) INTO member_count
  FROM member
  WHERE 'Member' = ANY(roles);

  -- Verificar si se excede el límite de 48 miembros con rol 'Member'
   IF (TG_OP = 'INSERT' OR (TG_OP = 'UPDATE' AND 'Member' = ANY(NEW.roles))) AND member_count >= 48 THEN
    RAISE EXCEPTION '48 member limit reached';
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION check_personal_referee()
RETURNS TRIGGER AS $$
BEGIN
  -- Verificar que solo se pueda asignar un referí personal a miembros con rol 'Member' en el array
  IF NEW.personal_referee IS NOT NULL AND NOT 'Member' = ANY(NEW.roles) THEN
    RAISE EXCEPTION 'personal referee is for members only';
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER personal_referee_trigger
BEFORE INSERT OR UPDATE ON member
FOR EACH ROW
EXECUTE FUNCTION check_personal_referee();

CREATE OR REPLACE FUNCTION set_referee_job_after_role()
RETURNS TRIGGER AS $$
BEGIN
  IF 'Referee' = ANY(NEW.roles) THEN
    UPDATE member SET job = 'Referee' WHERE id = NEW.id;
  END IF;
  RETURN NEW; -- Los triggers AFTER también deben devolver NEW o NULL
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_job_on_referee_role ON member;

CREATE TRIGGER set_job_on_referee_role_after
AFTER UPDATE OF roles ON member
FOR EACH ROW
WHEN (NOT 'Referee' = ANY(OLD.roles) AND 'Referee' = ANY(NEW.roles))
EXECUTE FUNCTION set_referee_job_after_role();

CREATE OR REPLACE FUNCTION check_job_requirements()
RETURNS TRIGGER AS $$
DECLARE
  required_role_job roles_enum;
  required_level_job INT;
  member_roles roles_enum[];
  member_level INT;
BEGIN
  -- Si se está limpiando el trabajo (asignando NULL), permitirlo.
  IF NEW.job IS NULL THEN
    RETURN NEW;
  END IF;

  -- Obtener el rol y nivel requerido para el trabajo que se intenta asignar.
  SELECT required_role, required_level
  INTO required_role_job, required_level_job
  FROM job
  WHERE name = NEW.job;

  -- Si el trabajo no existe, dejar que la restricción de clave foránea falle (si la tienes).
  IF NOT FOUND THEN
    RETURN NEW;
  END IF;

  -- Obtener los roles y nivel del miembro.
  SELECT roles, level
  INTO member_roles, member_level
  FROM member
  WHERE id = NEW.id;

  -- Verificar si el miembro cumple con el rol requerido.
  IF required_role_job IS NOT NULL AND NOT required_role_job = ANY(member_roles) THEN
    RAISE EXCEPTION 'El miembro no tiene el rol requerido (%) para el trabajo %.', required_role_job, NEW.job;
  END IF;

  -- Verificar si el miembro cumple con el nivel requerido.
  IF required_level_job IS NOT NULL AND member_level < required_level_job THEN
    RAISE EXCEPTION 'El miembro no tiene el nivel requerido (%) para el trabajo %.', required_level_job, NEW.job;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_member_job_requirements
BEFORE UPDATE OF job ON member
FOR EACH ROW
WHEN (OLD.job IS DISTINCT FROM NEW.job) -- Solo ejecutar si el trabajo está cambiando
EXECUTE FUNCTION check_job_requirements();

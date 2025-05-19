ALTER TABLE member
    ADD COLUMN special_balance INT NOT NULL DEFAULT 0;

-- varios grupos semanales de lb
CREATE TABLE week_group (
  id serial PRIMARY KEY,
  created_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
  is_active boolean NOT NULL DEFAULT true,
  member_limit int NOT NULL DEFAULT 10
);

-- miembro que pertenece a un grupo semanal
CREATE TABLE member_week_group (
  group_id int NOT NULL REFERENCES week_group(id) ON DELETE CASCADE,
  member bigint NOT NULL REFERENCES member(id) ON DELETE CASCADE,
  registered_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
  balance bigint NOT NULL DEFAULT 0,
  points int NOT NULL DEFAULT 0,
  gambles int NOT NULL DEFAULT 0,
  claimed_reward BOOL NOT NULL DEFAULT FALSE,
  PRIMARY KEY (group_id, member)
);

-- lb mensual que es afectado por cada lb semanal
-- funciona de forma global y no grupal
CREATE TABLE member_month_stats (
  member bigint NOT NULL REFERENCES member(id) ON DELETE CASCADE,
  month_start timestamp NOT NULL,
  special_balance int NOT NULL DEFAULT 0
);

-- Función para añadir a un miembro a un grupo semanal activo
CREATE OR REPLACE FUNCTION add_member_to_active_group(p_member_id bigint)
RETURNS boolean AS $$
DECLARE
    v_group_id int;
    v_group_member_count int;
    v_member_has_group boolean;
BEGIN
    -- Verificar si el miembro ya está en algún grupo semanal
    SELECT EXISTS(
        SELECT 1 FROM member_week_group
        WHERE member = p_member_id
    ) INTO v_member_has_group;

    -- Si el miembro ya está en un grupo, no hacemos nada
    IF v_member_has_group THEN
        RETURN false;
    END IF;

    -- Buscar un grupo activo con espacio disponible
    SELECT wg.id INTO v_group_id
    FROM week_group wg
    LEFT JOIN (
        SELECT group_id, COUNT(*) as member_count
        FROM member_week_group
        GROUP BY group_id
    ) counts ON wg.id = counts.group_id
    WHERE wg.is_active = true
      AND (counts.member_count IS NULL OR counts.member_count < wg.member_limit)
    ORDER BY
        CASE WHEN counts.member_count IS NULL THEN 0 ELSE counts.member_count END DESC
    LIMIT 1;

    -- Si no hay grupos activos con espacio, intentamos crear uno nuevo
    IF v_group_id IS NULL THEN
        INSERT INTO week_group (created_at, is_active, member_limit)
        VALUES (TIMEZONE('UTC', NOW()), true, 10)
        RETURNING id INTO v_group_id;
    END IF;

    -- Añadir miembro al grupo
    INSERT INTO member_week_group (group_id, member, balance, points, gambles)
    VALUES (v_group_id, p_member_id,
           (SELECT balance FROM member WHERE id = p_member_id),
           (SELECT points FROM member WHERE id = p_member_id),
           0);

    RETURN true;

EXCEPTION
    WHEN OTHERS THEN
        RAISE NOTICE 'Error al añadir miembro % al grupo: %', p_member_id, SQLERRM;
        RETURN false;
END;
$$ LANGUAGE plpgsql;

-- Función completa para actualizar estadísticas semanales
CREATE OR REPLACE FUNCTION update_week_stats()
RETURNS TRIGGER AS $$
DECLARE
    points_diff int := NEW.points - OLD.points;
    balance_diff bigint := NEW.balance - OLD.balance;
    current_week_start date := DATE_TRUNC('week', TIMEZONE('UTC', NOW()));
    v_group_id int;
    v_is_active boolean;
    v_added_to_group boolean;
BEGIN
    -- Verificar si el miembro está en algún grupo semanal activo
    SELECT mwg.group_id, wg.is_active INTO v_group_id, v_is_active
    FROM member_week_group mwg
    JOIN week_group wg ON mwg.group_id = wg.id
    WHERE mwg.member = NEW.id
    LIMIT 1;

    -- Si no está, lo añadimos a uno
    IF v_group_id IS NULL THEN
        SELECT add_member_to_active_group(NEW.id) INTO v_added_to_group;

        IF v_added_to_group THEN
            SELECT mwg.group_id, wg.is_active INTO v_group_id, v_is_active
            FROM member_week_group mwg
            JOIN week_group wg ON mwg.group_id = wg.id
            WHERE mwg.member = NEW.id
            LIMIT 1;
        ELSE
            RETURN NEW;
        END IF;
    END IF;

    IF NOT v_is_active THEN
        RETURN NEW;
    END IF;

    UPDATE member_week_group
    SET points = points + points_diff,
        balance = balance + balance_diff
    WHERE member = NEW.id AND group_id = v_group_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger para la tabla member
CREATE TRIGGER update_member_week_stats
AFTER UPDATE OF points, balance ON member
FOR EACH ROW
WHEN (OLD.points IS DISTINCT FROM NEW.points OR OLD.balance IS DISTINCT FROM NEW.balance)
EXECUTE FUNCTION update_week_stats();

-- Trigger para añadir nuevos miembros a un grupo activo
CREATE OR REPLACE FUNCTION assign_new_member_to_group()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM add_member_to_active_group(NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Crear el trigger
CREATE TRIGGER assign_member_to_group
AFTER INSERT ON member
FOR EACH ROW
EXECUTE FUNCTION assign_new_member_to_group();

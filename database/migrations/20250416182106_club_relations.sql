CREATE TABLE agent_slots (
    club bigint REFERENCES club (id) ON DELETE CASCADE,
    agent_range int,
    occupied boolean DEFAULT FALSE,
    PRIMARY KEY (club, agent_range)
);

CREATE TABLE club_req (
    club bigint REFERENCES club (id) ON DELETE CASCADE,
    balance bigint NOT NULL DEFAULT 0,
    points int NOT NULL DEFAULT 0
);

CREATE FUNCTION create_club (
    leader bigint,
    club_name varchar(255),
    leader_role_name varchar(255),
    agent_role_name varchar(255),
    member_role_name varchar(255)
)
RETURNS BIGINT
AS $$
DECLARE
    club_id bigint;
    agent_limit int;
BEGIN
    -- Evita nombres duplicados
    IF EXISTS (SELECT 1 FROM club WHERE name = club_name) THEN
        RAISE EXCEPTION 'Ya existe un club con el nombre "%".', club_name
            USING ERRCODE = 'unique_violation';
    END IF;

    -- Crea el club
    INSERT INTO club (leader, name)
    VALUES (leader, club_name)
    RETURNING id INTO club_id;

    -- Crea el rol de líder
    INSERT INTO club_role (club, tr_key, authority, authority_id, perms)
    VALUES (club_id, leader_role_name, 100, 'Leader', '{ManageRoles, ManageMembers, ManageBank, ManageClub, InviteMembers}');

    INSERT INTO club_limits (club, role_name, member_limit)
    VALUES (club_id, leader_role_name, 1);

    -- Asigna al líder como miembro del club si existe
    IF leader IS NOT NULL THEN
        INSERT INTO club_member (club, member, role_name)
        VALUES (club_id, leader, leader_role_name);

        -- Loguea la asignación inicial del rol del líder
        PERFORM log_club_role(club_id, leader, leader_role_name, NULL, NULL);
    END IF;

    -- Crea el rol de miembro
    INSERT INTO club_role (club, tr_key, authority, authority_id)
    VALUES (club_id, member_role_name, 10, 'Member');

    INSERT INTO club_limits (club, role_name, member_limit)
    VALUES (club_id, member_role_name, 48);

    INSERT INTO club_role (club, tr_key, authority, authority_id, perms)
    VALUES (club_id, agent_role_name, 70, 'Agent', '{ManageBank, InviteMembers}');

    INSERT INTO club_limits (club, role_name, member_limit)
    VALUES (club_id, agent_role_name, 101)
    RETURNING member_limit INTO agent_limit;

    INSERT INTO agent_slots (club, agent_range)
    SELECT club_id, generate_series(0, agent_limit);

    INSERT INTO club_stl_rules (club) VALUES (club_id);

    INSERT INTO club_req (club) VALUES (club_id);

    RETURN club_id;
EXCEPTION
    WHEN OTHERS THEN
        RAISE;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS agent_relation (
    club bigint,
    member bigint,
    agent bigint,
    created_at timestamp NOT NULL DEFAULT TIMEZONE('UTC', NOW()),
    FOREIGN KEY (club, member) REFERENCES club_member (club, member) ON DELETE CASCADE,
    PRIMARY KEY (club, member, agent)
);

SELECT
    create_club (1323132058184192030, 'Kakerou', 'leader', 'referee', 'member');

SELECT log_club_role(1, 1323132058184192030, 'leader', NULL, NULL);

UPDATE club SET renameable = FALSE, deleteable = FALSE WHERE name = 'Kakerou';

-- Función para asignar roles a miembros
CREATE OR REPLACE FUNCTION assign_club_role(
    p_club_id BIGINT,
    p_member_id BIGINT,
    p_role_name VARCHAR(255)
)
RETURNS BOOLEAN AS $$
DECLARE
    v_exists BOOLEAN;
    v_role_exists BOOLEAN;
BEGIN
    -- Verificar si el miembro existe en el club
    SELECT EXISTS (
        SELECT 1 FROM club_member 
        WHERE club = p_club_id AND member = p_member_id
    ) INTO v_exists;
    
    IF NOT v_exists THEN
        RAISE EXCEPTION 'El miembro % no pertenece al club %', p_member_id, p_club_id;
    END IF;
    
    -- Verificar si el rol existe en el club
    SELECT EXISTS (
        SELECT 1 FROM club_role 
        WHERE club = p_club_id AND tr_key = p_role_name
    ) INTO v_role_exists;
    
    IF NOT v_role_exists THEN
        RAISE EXCEPTION 'El rol % no existe en el club %', p_role_name, p_club_id;
    END IF;
    
    -- Actualizar el rol del miembro
    UPDATE club_member 
    SET role_name = p_role_name
    WHERE club = p_club_id AND member = p_member_id;
    
    RETURN TRUE;
EXCEPTION
    WHEN OTHERS THEN
        RAISE;
END;
$$ LANGUAGE plpgsql;

-- Función para asignar rol de agente con un rango específico
CREATE OR REPLACE FUNCTION assign_agent_role(
    p_club_id BIGINT,
    p_member_id BIGINT,
    p_agent_role_name VARCHAR(255),
    p_requested_range INT
)
RETURNS INT AS $$
DECLARE
    v_exists BOOLEAN;
    v_role_exists BOOLEAN;
    v_is_agent BOOLEAN;
    v_assigned_range INT;
    v_current_range INT;
BEGIN
    -- Verificar si el miembro existe en el club
    SELECT EXISTS (
        SELECT 1 FROM club_member 
        WHERE club = p_club_id AND member = p_member_id
    ) INTO v_exists;
    
    IF NOT v_exists THEN
        RAISE EXCEPTION 'El miembro % no pertenece al club %', p_member_id, p_club_id;
    END IF;
    
    -- Verificar si el rol existe y es de tipo agente
    SELECT 
        EXISTS(SELECT 1 FROM club_role WHERE club = p_club_id AND tr_key = p_agent_role_name),
        (SELECT authority_id = 'Agent' FROM club_role WHERE club = p_club_id AND tr_key = p_agent_role_name)
    INTO v_role_exists, v_is_agent;
    
    IF NOT v_role_exists THEN
        RAISE EXCEPTION 'El rol % no existe en el club %', p_agent_role_name, p_club_id;
    END IF;
    
    IF NOT v_is_agent THEN
        RAISE EXCEPTION 'El rol % no es de tipo agente', p_agent_role_name;
    END IF;
    
    -- Verificar si el miembro ya tiene un rango asignado
    SELECT agent_range INTO v_current_range
    FROM club_member
    WHERE club = p_club_id AND member = p_member_id;
    
    -- Si ya tiene un rango y es diferente al solicitado, liberarlo
    IF v_current_range IS NOT NULL AND v_current_range != p_requested_range THEN
        UPDATE agent_slots
        SET occupied = FALSE
        WHERE club = p_club_id AND agent_range = v_current_range;
    END IF;
    
    -- Intentar asignar el rango solicitado si está disponible
    UPDATE agent_slots
    SET occupied = TRUE
    WHERE club = p_club_id 
      AND agent_range = p_requested_range 
      AND occupied = FALSE
    RETURNING agent_range INTO v_assigned_range;
    
    -- Si el rango solicitado no está disponible, buscar el más cercano disponible
    IF v_assigned_range IS NULL THEN
        -- Primero intentar con rangos superiores
        SELECT agent_range INTO v_assigned_range
        FROM agent_slots
        WHERE club = p_club_id 
          AND occupied = FALSE
          AND agent_range > p_requested_range
        ORDER BY agent_range
        LIMIT 1;
        
        -- Si no hay rangos superiores disponibles, intentar con rangos inferiores
        IF v_assigned_range IS NULL THEN
            SELECT agent_range INTO v_assigned_range
            FROM agent_slots
            WHERE club = p_club_id 
              AND occupied = FALSE
              AND agent_range < p_requested_range
            ORDER BY agent_range DESC
            LIMIT 1;
        END IF;
        
        -- Si se encontró un rango alternativo, marcarlo como ocupado
        IF v_assigned_range IS NOT NULL THEN
            UPDATE agent_slots
            SET occupied = TRUE
            WHERE club = p_club_id AND agent_range = v_assigned_range;
        ELSE
            RAISE EXCEPTION 'No hay rangos de agente disponibles en el club %', p_club_id;
        END IF;
    END IF;
    
    -- Actualizar el miembro con el rol de agente y el rango asignado
    UPDATE club_member 
    SET role_name = p_agent_role_name,
        agent_range = v_assigned_range
    WHERE club = p_club_id AND member = p_member_id;
    
    RETURN v_assigned_range;
EXCEPTION
    WHEN OTHERS THEN
        RAISE;
END;
$$ LANGUAGE plpgsql;

-- Insert initial administrator user
DO $$
DECLARE
    v_admin_id UUID;
    v_role_id UUID;
BEGIN
    -- 1. Insert or Get Administrator
    INSERT INTO user_administrators (email, password_hash, first_name, last_name)
    VALUES (
        'admin@email.com',
        '$argon2id$v=19$m=19456,t=2,p=1$SNdmdsg99eWhTp1gfhgQng$Ew5Zwb7you3ebqt849JzCR3+sKbzn8jW7DIsE3OMo7c', -- Password: "123123123"
        'Admin',
        'User'
    )
    ON CONFLICT (email) DO UPDATE 
    SET updated_at = NOW() -- Dummy update to return ID
    RETURNING id INTO v_admin_id;

    -- If no ID returned (because it already existed and we didn't use RETURNING correctly with ON CONFLICT UPDATE in some postgres versions without changes), fetch it.
    IF v_admin_id IS NULL THEN
        SELECT id INTO v_admin_id FROM user_administrators WHERE email = 'admin@email.com';
    END IF;

    -- 2. Insert or Get "Super Admin" Role
    -- Note: roles has unique index on (name, scope, coalesce(group_id...))
    SELECT id INTO v_role_id FROM roles 
    WHERE name = 'Super Admin' AND scope = 'ADMINISTRATOR' AND group_id IS NULL;

    IF v_role_id IS NULL THEN
        INSERT INTO roles (name, description, scope, group_id)
        VALUES ('Super Admin', 'Initial super administrator role with full access', 'ADMINISTRATOR', NULL)
        RETURNING id INTO v_role_id;
    END IF;

    -- 3. Assign role to administrator
    INSERT INTO administrator_roles (administrator_id, role_id)
    VALUES (v_admin_id, v_role_id)
    ON CONFLICT (administrator_id, role_id) DO NOTHING;

    -- 4. Assign wildcard permission to Super Admin role
    INSERT INTO role_permissions (role_id, permission)
    SELECT v_role_id, '*'
    WHERE NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = v_role_id AND permission = '*');

END $$;

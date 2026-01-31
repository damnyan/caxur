CREATE TABLE IF NOT EXISTS administrator_roles (
    administrator_id UUID NOT NULL REFERENCES user_administrators(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (administrator_id, role_id)
);

CREATE INDEX idx_administrator_roles_admin_id ON administrator_roles(administrator_id);
CREATE INDEX idx_administrator_roles_role_id ON administrator_roles(role_id);

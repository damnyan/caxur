CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    scope VARCHAR(50) NOT NULL DEFAULT 'ADMINISTRATOR',
    group_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_roles_name ON roles(name);
CREATE INDEX idx_roles_scope ON roles(scope);
CREATE INDEX idx_roles_group_id ON roles(group_id);
CREATE UNIQUE INDEX idx_roles_name_scope_group ON roles(name, scope, COALESCE(group_id, '00000000-0000-0000-0000-000000000000'));
CREATE INDEX idx_roles_created_at ON roles(created_at DESC);

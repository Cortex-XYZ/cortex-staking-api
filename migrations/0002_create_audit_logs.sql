CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    actor_api_key_id UUID
        REFERENCES api_keys(id)
        ON DELETE SET NULL,

    actor_organization_id UUID
        REFERENCES organizations(id)
        ON DELETE SET NULL,

    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID,

    ip_address TEXT,
    request_id TEXT,

    old_values JSONB,
    new_values JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_logs_actor_api_key_id_idx
    ON audit_logs (actor_api_key_id);

CREATE INDEX audit_logs_actor_organization_id_idx
    ON audit_logs (actor_organization_id);

CREATE INDEX audit_logs_resource_idx
    ON audit_logs (resource_type, resource_id);

CREATE INDEX audit_logs_action_idx
    ON audit_logs (action);

CREATE INDEX audit_logs_created_at_idx
    ON audit_logs (created_at DESC);
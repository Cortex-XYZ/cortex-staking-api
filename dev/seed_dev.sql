INSERT INTO organizations (name, kind)
VALUES
    ('Cortex Global', 'cortex'),
    ('Dev Partner', 'partner')
ON CONFLICT DO NOTHING;

INSERT INTO api_keys (
    owner_type,
    organization_id,
    name,
    key_prefix,
    key_hash,
    rate_limit_per_minute
)
SELECT
    'organization',
    id,
    'Cortex Dev Admin Key',
    'ctx_dev_cortex',
    encode(digest('ctx_dev_cortex.secret', 'sha256'), 'hex'),
    1000
FROM organizations
WHERE name = 'Cortex Global'
ON CONFLICT (key_prefix) DO NOTHING;

INSERT INTO api_key_scopes (api_key_id, scope)
SELECT id, 'admin:*'
FROM api_keys
WHERE key_prefix = 'ctx_dev_cortex'
ON CONFLICT DO NOTHING;

INSERT INTO api_keys (
    owner_type,
    organization_id,
    name,
    key_prefix,
    key_hash,
    rate_limit_per_minute
)
SELECT
    'organization',
    id,
    'Partner Dev Key',
    'ctx_dev_partner',
    encode(digest('ctx_dev_partner.secret', 'sha256'), 'hex'),
    120
FROM organizations
WHERE name = 'Dev Partner'
ON CONFLICT (key_prefix) DO NOTHING;

INSERT INTO api_key_scopes (api_key_id, scope)
SELECT id, 'admin'
FROM api_keys
WHERE key_prefix = 'ctx_dev_cortex'
ON CONFLICT DO NOTHING;

INSERT INTO api_key_scopes (api_key_id, scope)
SELECT id, 'read'
FROM api_keys
WHERE key_prefix = 'ctx_dev_partner'
ON CONFLICT DO NOTHING;

-- ============================================================
-- Development User
-- ============================================================

INSERT INTO users (
    email,
    wallet_address,
    status,
    key_limit,
    rate_limit_tier
)
VALUES (
    'dev-user@example.com',
    '0x1234567890abcdef1234567890abcdef12345678',
    'active',
    2,
    'free'
);
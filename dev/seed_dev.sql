-- ============================================================
-- Cortex Staking API
-- Development Seed Data
--
-- WARNING:
-- This file creates known plaintext API keys.
-- Use only for local development and automated testing.
-- Never run this seed file in production.
-- ============================================================

CREATE EXTENSION IF NOT EXISTS pgcrypto;

BEGIN;

-- ============================================================
-- Organizations
-- ============================================================

INSERT INTO organizations (
    name,
    kind,
    status
)
VALUES
    (
        'Cortex Global',
        'cortex',
        'active'
    ),
    (
        'Dev Partner',
        'partner',
        'active'
    )
ON CONFLICT DO NOTHING;


-- ============================================================
-- Cortex Development Admin Key
--
-- Plaintext token:
-- ctx_dev_cortex.secret
-- ============================================================

INSERT INTO api_keys (
    owner_type,
    organization_id,
    user_id,
    name,
    key_prefix,
    key_hash,
    status,
    rate_limit_per_minute,
    expires_at,
    revoked_at
)
SELECT
    'organization',
    organizations.id,
    NULL,
    'Cortex Dev Admin Key',
    'ctx_dev_cortex',
    encode(
        digest('ctx_dev_cortex.secret', 'sha256'),
        'hex'
    ),
    'active',
    1000,
    NULL,
    NULL
FROM organizations
WHERE organizations.name = 'Cortex Global'
ON CONFLICT (key_prefix) DO UPDATE
SET
    owner_type = EXCLUDED.owner_type,
    organization_id = EXCLUDED.organization_id,
    user_id = NULL,
    name = EXCLUDED.name,
    key_hash = EXCLUDED.key_hash,
    status = 'active',
    rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
    expires_at = NULL,
    revoked_at = NULL,
    updated_at = now();


-- ============================================================
-- Cortex Bruno Admin Key
--
-- Plaintext token:
-- ctx_dev_bruno.secret
-- ============================================================

INSERT INTO api_keys (
    owner_type,
    organization_id,
    user_id,
    name,
    key_prefix,
    key_hash,
    status,
    rate_limit_per_minute,
    expires_at,
    revoked_at
)
SELECT
    'organization',
    organizations.id,
    NULL,
    'Cortex Bruno Admin Key',
    'ctx_dev_bruno',
    encode(
        digest('ctx_dev_bruno.secret', 'sha256'),
        'hex'
    ),
    'active',
    1000,
    NULL,
    NULL
FROM organizations
WHERE organizations.name = 'Cortex Global'
ON CONFLICT (key_prefix) DO UPDATE
SET
    owner_type = EXCLUDED.owner_type,
    organization_id = EXCLUDED.organization_id,
    user_id = NULL,
    name = EXCLUDED.name,
    key_hash = EXCLUDED.key_hash,
    status = 'active',
    rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
    expires_at = NULL,
    revoked_at = NULL,
    updated_at = now();


-- ============================================================
-- Partner Development Key
--
-- Plaintext token:
-- ctx_dev_partner.secret
-- ============================================================

INSERT INTO api_keys (
    owner_type,
    organization_id,
    user_id,
    name,
    key_prefix,
    key_hash,
    status,
    rate_limit_per_minute,
    expires_at,
    revoked_at
)
SELECT
    'organization',
    organizations.id,
    NULL,
    'Partner Dev Key',
    'ctx_dev_partner',
    encode(
        digest('ctx_dev_partner.secret', 'sha256'),
        'hex'
    ),
    'active',
    120,
    NULL,
    NULL
FROM organizations
WHERE organizations.name = 'Dev Partner'
ON CONFLICT (key_prefix) DO UPDATE
SET
    owner_type = EXCLUDED.owner_type,
    organization_id = EXCLUDED.organization_id,
    user_id = NULL,
    name = EXCLUDED.name,
    key_hash = EXCLUDED.key_hash,
    status = 'active',
    rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
    expires_at = NULL,
    revoked_at = NULL,
    updated_at = now();


-- ============================================================
-- Reset development scopes
--
-- Removing existing scopes first keeps repeated seed runs
-- deterministic.
-- ============================================================

DELETE FROM api_key_scopes
WHERE api_key_id IN (
    SELECT id
    FROM api_keys
    WHERE key_prefix IN (
        'ctx_dev_cortex',
        'ctx_dev_bruno',
        'ctx_dev_partner'
    )
);


-- ============================================================
-- Cortex Admin Scopes
-- ============================================================

INSERT INTO api_key_scopes (api_key_id, scope)
SELECT api_keys.id, scopes.scope
FROM api_keys
CROSS JOIN (
    VALUES
        ('admin'),
        ('admin:*')
) AS scopes(scope)
WHERE api_keys.key_prefix IN (
    'ctx_dev_cortex',
    'ctx_dev_bruno'
)
ON CONFLICT DO NOTHING;


-- ============================================================
-- Partner Scopes
-- ============================================================

INSERT INTO api_key_scopes (
    api_key_id,
    scope
)
SELECT
    api_keys.id,
    'read'
FROM api_keys
WHERE api_keys.key_prefix = 'ctx_dev_partner'
ON CONFLICT DO NOTHING;

INSERT INTO api_key_scopes (
    api_key_id,
    scope
)
SELECT
    api_keys.id,
    'write'
FROM api_keys
WHERE api_keys.key_prefix = 'ctx_dev_partner'
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
)
ON CONFLICT DO NOTHING;

COMMIT;


-- ============================================================
-- Verification output
-- ============================================================

SELECT
    organizations.name AS organization_name,
    organizations.kind AS organization_kind,
    api_keys.id AS api_key_id,
    api_keys.name AS api_key_name,
    api_keys.key_prefix,
    api_keys.status,
    api_keys.rate_limit_per_minute
FROM api_keys
LEFT JOIN organizations
    ON organizations.id = api_keys.organization_id
WHERE api_keys.key_prefix IN (
    'ctx_dev_cortex',
    'ctx_dev_bruno',
    'ctx_dev_partner'
)
ORDER BY api_keys.key_prefix;

SELECT
    api_keys.key_prefix,
    api_key_scopes.scope
FROM api_keys
JOIN api_key_scopes
    ON api_key_scopes.api_key_id = api_keys.id
WHERE api_keys.key_prefix IN (
    'ctx_dev_cortex',
    'ctx_dev_bruno',
    'ctx_dev_partner'
)
ORDER BY
    api_keys.key_prefix,
    api_key_scopes.scope;


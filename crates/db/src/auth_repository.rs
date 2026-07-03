use cortex_auth::model::{AuthContext, OrganizationKind, Scope};
use sqlx::{PgPool, Row};

pub async fn authenticate_api_key(
    db: &PgPool,
    token: &str,
) -> Result<Option<AuthContext>, sqlx::Error> {
    let Some(key_prefix) = token.split('.').next() else {
        return Ok(None);
    };

    let rows = sqlx::query(
        r#"
        SELECT
            ak.id::text AS api_key_id,
            ak.owner_type,
            ak.organization_id::text AS organization_id,
            o.kind AS organization_kind,
            ak.user_id::text AS user_id,
            ak.rate_limit_per_minute,
            aks.scope
        FROM api_keys ak
        LEFT JOIN organizations o ON o.id = ak.organization_id
        LEFT JOIN api_key_scopes aks ON aks.api_key_id = ak.id
        WHERE ak.key_prefix = $1
          AND ak.key_hash = encode(digest($2, 'sha256'), 'hex')
          AND ak.status = 'active'
          AND (ak.expires_at IS NULL OR ak.expires_at > now())
        "#,
    )
    .bind(key_prefix)
    .bind(token)
    .fetch_all(db)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let first = &rows[0];

    let owner_type: String = first.try_get("owner_type")?;
    let organization_kind_raw: Option<String> = first.try_get("organization_kind")?;

    let organization_kind = match (owner_type.as_str(), organization_kind_raw.as_deref()) {
        ("user", _) => OrganizationKind::User,
        ("organization", Some("cortex")) => OrganizationKind::Cortex,
        ("organization", Some("partner")) => OrganizationKind::Partner,
        _ => return Ok(None),
    };

    let mut scopes = Vec::new();

    for row in rows.iter() {
        let scope: Option<String> = row.try_get("scope")?;

        if let Some(scope) = scope {
            if let Some(scope) = Scope::from_db_value(&scope) {
                scopes.push(scope);
            }
        }
    }

    Ok(Some(AuthContext {
        api_key_id: first.try_get("api_key_id")?,
        organization_id: first.try_get("organization_id")?,
        organization_kind,
        user_id: first.try_get("user_id")?,
        scopes,
    }))
}

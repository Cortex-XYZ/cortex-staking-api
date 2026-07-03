#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrganizationKind {
    Cortex,
    Partner,
    User,
}

impl OrganizationKind {
    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "cortex" => Some(Self::Cortex),
            "partner" => Some(Self::Partner),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Admin,
    Read,
    Write,
}

impl Scope {
    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Self::Admin),
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub api_key_id: String,
    pub organization_id: Option<String>,
    pub user_id: Option<String>,
    pub organization_kind: OrganizationKind,
    pub scopes: Vec<Scope>,
}

impl AuthContext {
    pub fn is_cortex_admin(&self) -> bool {
        self.organization_kind == OrganizationKind::Cortex && self.scopes.contains(&Scope::Admin)
    }

    pub fn is_partner(&self) -> bool {
        self.organization_kind == OrganizationKind::Partner
    }

    pub fn is_user(&self) -> bool {
        self.organization_kind == OrganizationKind::User
    }

    pub fn has_scope(&self, scope: Scope) -> bool {
        self.scopes.contains(&scope) || self.scopes.contains(&Scope::Admin)
    }

    pub fn can_read(&self) -> bool {
        self.has_scope(Scope::Read)
    }

    pub fn can_write(&self) -> bool {
        self.has_scope(Scope::Write)
    }
}

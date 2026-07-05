use cortex_db::{
    pagination::DbPagination,
    user_repository::{self, PaginatedUsers, UserRecord},
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct UpdateUserServiceInput {
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub status: Option<String>,
    pub key_limit: Option<i32>,
    pub rate_limit_tier: Option<String>,
}

pub async fn list_users(
    db: &PgPool,
    pagination: DbPagination,
) -> Result<PaginatedUsers, sqlx::Error> {
    user_repository::list_users(db, pagination).await
}

pub async fn get_user_by_id(
    db: &PgPool,
    user_id: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    user_repository::get_user_by_id(db, user_id).await
}

pub async fn update_user(
    db: &PgPool,
    user_id: &str,
    input: UpdateUserServiceInput,
) -> Result<Option<UserRecord>, sqlx::Error> {
    user_repository::update_user(
        db,
        user_id,
        user_repository::UpdateUserInput {
            email: input.email,
            wallet_address: input.wallet_address,
            status: input.status,
            key_limit: input.key_limit,
            rate_limit_tier: input.rate_limit_tier,
        },
    )
    .await
}

pub async fn delete_user(
    db: &PgPool,
    user_id: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    user_repository::soft_delete_user(db, user_id).await
}
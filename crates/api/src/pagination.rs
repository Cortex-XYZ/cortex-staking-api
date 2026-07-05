use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: i64,
    pub page_size: i64,
    pub offset: i64,
    pub sort: Option<String>,
    pub direction: SortDirection,
}

#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl PaginationQuery {
    pub fn into_pagination(self) -> Pagination {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(100).clamp(1, 500);
        let offset = (page - 1) * page_size;

        let direction = match self.direction.as_deref() {
            Some("asc") => SortDirection::Asc,
            _ => SortDirection::Desc,
        };

        Pagination {
            page,
            page_size,
            offset,
            sort: self.sort,
            direction,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
    pub total_pages: i64,
}
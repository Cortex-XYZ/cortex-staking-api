#[derive(Debug, Clone)]
pub struct DbPagination {
    pub limit: i64,
    pub offset: i64,
    pub sort_column: String,
    pub sort_direction: String,
}

impl DbPagination {
    pub fn new(
        limit: i64,
        offset: i64,
        sort_column: impl Into<String>,
        sort_direction: impl Into<String>,
    ) -> Self {
        Self {
            limit,
            offset,
            sort_column: sort_column.into(),
            sort_direction: sort_direction.into(),
        }
    }
}
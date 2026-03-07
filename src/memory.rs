use supabase_wrappers::prelude::*;
use pgrx::pg_sys::panic::ErrorReport;
use pgrx::PgSqlErrorCode;
use thiserror::Error;

#[wrappers_fdw(version = "0.1.0", author = "me", error_type = "MemoryError")]
struct Memory {
}

#[derive(Error, Debug)]
enum MemoryError {
}

impl From<MemoryError> for ErrorReport {
    fn from(value: MemoryError) -> Self {
        ErrorReport::new(
            PgSqlErrorCode::ERRCODE_FDW_ERROR,
            value.to_string(),
            "github",
        )
    }
}

type Result<T> = std::result::Result<T, MemoryError>;

impl ForeignDataWrapper<MemoryError> for Memory {
    fn new(_server: ForeignServer) -> Result<Self>
    where
        Self: Sized {
        todo!()
    }

    fn begin_scan(
        &mut self,
        _quals: &[Qual],
        _columns: &[Column],
        _sorts: &[Sort],
        _limit: &Option<Limit>,
        _options: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        todo!()
    }

    fn iter_scan(&mut self, _row: &mut Row) -> Result<Option<()>> {
        todo!()
    }

    fn end_scan(&mut self) -> Result<()> {
        todo!()
    }
}

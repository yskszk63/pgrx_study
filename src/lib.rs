use pgrx::PgSqlErrorCode;
use pgrx::pg_sys::panic::ErrorReport;
use supabase_wrappers::prelude::*;

::pgrx::pg_module_magic!(name, version);

#[wrappers_fdw(
    version = "0.1.0",
    author = "me",
    error_type = "MyTestError",
)]
struct MyTest {}

enum MyTestError {}

impl From<MyTestError> for ErrorReport {
    fn from(_value: MyTestError) -> Self {
        ErrorReport::new(PgSqlErrorCode::ERRCODE_FDW_ERROR, "", "")
    }
}

type MyTestResult<T> = Result<T, MyTestError>;

impl ForeignDataWrapper<MyTestError> for MyTest {
    fn new(_server: ForeignServer) -> Result<Self, MyTestError>
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
    ) -> Result<(), MyTestError> {
        todo!()
    }

    fn iter_scan(&mut self, _row: &mut Row) -> Result<Option<()>, MyTestError> {
        todo!()
    }

    fn end_scan(&mut self) -> Result<(), MyTestError> {
        todo!()
    }
}

use std::os::unix::fs::MetadataExt;
use std::{fs, io};

use pgrx::pg_sys::panic::ErrorReport;
use pgrx::PgSqlErrorCode;
use supabase_wrappers::prelude::*;
use thiserror::Error;

// NOTE: Atomic doesn't guarantee it!

#[wrappers_fdw(version = "0.1.0", author = "me", error_type = "DirError")]
struct Dir {
    cursor: Option<fs::ReadDir>,
}

#[derive(Error, Debug)]
enum DirError {
    #[error("No option")]
    NoOption,

    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("Not implemented.")]
    NotImplemented,
}

impl From<DirError> for ErrorReport {
    fn from(value: DirError) -> Self {
        ErrorReport::new(
            PgSqlErrorCode::ERRCODE_FDW_ERROR,
            value.to_string(),
            "github",
        )
    }
}

type Result<T> = std::result::Result<T, DirError>;

impl ForeignDataWrapper<DirError> for Dir {
    fn new(_server: ForeignServer) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self { cursor: None })
    }

    fn begin_scan(
        &mut self,
        _quals: &[Qual],
        _columns: &[Column],
        _sorts: &[Sort],
        _limit: &Option<Limit>,
        options: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let Some(path) = options.get("dir") else {
            return Err(DirError::NoOption);
        };

        let entries = fs::read_dir(path)?;
        self.cursor = Some(entries);
        Ok(())
    }

    fn iter_scan(&mut self, row: &mut Row) -> Result<Option<()>> {
        let Some(cursor) = &mut self.cursor else {
            return Ok(None);
        };

        loop {
            let Some(entry) = cursor.next() else {
                return Ok(None);
            };
            let entry = entry?;
            let metadata = entry.metadata()?;

            if !metadata.is_file() {
                continue;
            }

            row.push(
                "name",
                Some(Cell::String(entry.path().to_string_lossy().to_string())),
            );
            row.push(
                "mode",
                Some(Cell::String(format!("{:04o}", metadata.mode()))),
            );
            return Ok(Some(()));
        }
    }

    fn end_scan(&mut self) -> Result<()> {
        self.cursor = None;
        Ok(())
    }

    fn begin_modify(&mut self, _options: &std::collections::HashMap<String, String>) -> Result<()> {
        report_notice("begin_modify");
        Err(DirError::NotImplemented)
    }

    fn insert(&mut self, row: &Row) -> Result<()> {
        report_notice(&format!("insert {:?}", row));
        Err(DirError::NotImplemented)
    }

    fn update(&mut self, rowid: &Cell, new_row: &Row) -> Result<()> {
        report_notice(&format!("update {:?} {:?}", rowid, new_row));
        Err(DirError::NotImplemented)
    }

    fn delete(&mut self, rowid: &Cell) -> Result<()> {
        report_notice(&format!("delete {:?}", rowid));
        Err(DirError::NotImplemented)
    }

    fn end_modify(&mut self) -> Result<()> {
        report_notice("end_modify");
        Ok(())
    }
}

use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::{fs, io};

use pgrx::pg_sys::panic::ErrorReport;
use pgrx::PgSqlErrorCode;
use supabase_wrappers::prelude::*;
use thiserror::Error;

// NOTE: Atomic doesn't guarantee it!

#[wrappers_fdw(version = "0.1.0", author = "me", error_type = "DirError")]
struct Dir {
    cursor: Option<fs::ReadDir>,
    dir: Option<String>,
}

#[derive(Error, Debug)]
enum DirError {
    #[error("No option")]
    NoOption,

    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("Path required")]
    PathRequired,

    #[error("Must not contains dir")]
    MustNotContainsDir,

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
        Ok(Self {
            cursor: None,
            dir: None,
        })
    }

    fn begin_scan(
        &mut self,
        _quals: &[Qual],
        _columns: &[Column],
        _sorts: &[Sort],
        _limit: &Option<Limit>,
        options: &HashMap<String, String>,
    ) -> Result<()> {
        report_notice(&format!("update {:?}", _columns));

        let Some(path) = options.get("dir") else {
            return Err(DirError::NoOption);
        };

        let entries = fs::read_dir(path)?;
        self.cursor = Some(entries);
        self.dir = Some(path.to_owned());
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
        self.dir = None;
        Ok(())
    }

    fn begin_modify(&mut self, options: &HashMap<String, String>) -> Result<()> {
        let Some(path) = options.get("dir") else {
            return Err(DirError::NoOption);
        };

        self.dir = Some(path.to_owned());
        Ok(())
    }

    fn insert(&mut self, row: &Row) -> Result<()> {
        let Some(dir) = &self.dir else {
            return Err(DirError::NoOption);
        };

        let row = row
            .cols
            .iter()
            .cloned()
            .zip(&row.cells)
            .collect::<HashMap<_, _>>();
        let Some(&Some(Cell::String(path))) = row.get("path") else {
            return Err(DirError::PathRequired);
        };

        let dir = Path::new(dir);
        let path = dir.join(path);
        if path.parent() != Some(dir) {
            return Err(DirError::MustNotContainsDir);
        };

        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?;
        // TODO mode
        Ok(())
    }

    fn update(&mut self, rowid: &Cell, new_row: &Row) -> Result<()> {
        report_notice(&format!("update {:?} {:?}", rowid, new_row));
        Err(DirError::NotImplemented)
    }

    fn delete(&mut self, rowid: &Cell) -> Result<()> {
        report_notice(&format!("update {:?}", rowid));
        let Some(dir) = &self.dir else {
            return Err(DirError::NoOption);
        };

        let Cell::String(path) = rowid else {
            return Err(DirError::PathRequired);
        };

        let dir = Path::new(dir);
        let path = dir.join(path);
        if path.parent() != Some(dir) {
            return Err(DirError::MustNotContainsDir);
        };

        fs::remove_file(path)?;
        Ok(())
    }

    fn end_modify(&mut self) -> Result<()> {
        self.cursor = None;
        self.dir = None;
        Ok(())
    }
}

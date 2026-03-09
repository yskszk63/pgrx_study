use std::collections::HashMap;
use std::ffi::c_uint;
use std::num::ParseIntError;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, StripPrefixError};
use std::{fs, io};

use pgrx::pg_sys::panic::ErrorReport;
use pgrx::PgSqlErrorCode;
use rustix::fs::{fchmod, Mode};
use supabase_wrappers::prelude::*;
use thiserror::Error;

// NOTE: Atomic doesn't guarantee it!

struct Scan {
    cursor: fs::ReadDir,
    dir: String,
    columns: Vec<Column>,
}

impl From<Scan> for State {
    fn from(value: Scan) -> Self {
        State::Scan(value)
    }
}

struct Update {
    dir: String,
}

impl From<Update> for State {
    fn from(value: Update) -> Self {
        State::Update(value)
    }
}

enum State {
    Scan(Scan),
    Update(Update),
}

#[wrappers_fdw(version = "0.1.0", author = "me", error_type = "DirError")]
struct Dir {
    state: Option<State>,
}

#[derive(Error, Debug)]
enum DirError {
    #[error("No option")]
    NoOption,

    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Errno(#[from] rustix::io::Errno),

    #[error("{0}")]
    StripPrefix(#[from] StripPrefixError),

    #[error("{0}")]
    ParseInt(#[from] ParseIntError),

    #[error("Invalid mode")]
    InvalidMode,

    #[error("Path required")]
    PathRequired,

    #[error("Must not contains dir")]
    MustNotContainsDir,
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

fn mode_from_str(v: &str) -> Result<Mode> {
    let raw = c_uint::from_str_radix(v, 8)?;
    Mode::from_bits(raw).ok_or_else(|| DirError::InvalidMode)
}

impl ForeignDataWrapper<DirError> for Dir {
    fn new(_server: ForeignServer) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self { state: None })
    }

    fn begin_scan(
        &mut self,
        _quals: &[Qual],
        columns: &[Column],
        _sorts: &[Sort],
        _limit: &Option<Limit>,
        options: &HashMap<String, String>,
    ) -> Result<()> {
        let Some(path) = options.get("dir") else {
            return Err(DirError::NoOption);
        };

        let entries = fs::read_dir(path)?;
        self.state = Some(
            Scan {
                cursor: entries,
                dir: path.to_owned(),
                columns: columns.to_owned(),
            }
            .into(),
        );
        Ok(())
    }

    fn iter_scan(&mut self, row: &mut Row) -> Result<Option<()>> {
        let Some(State::Scan(Scan {
            cursor,
            columns,
            dir,
        })) = &mut self.state
        else {
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

            let dir = Path::new(dir);
            for col in columns {
                match col.name.as_ref() {
                    "path" => {
                        row.push(
                            "path",
                            Some(Cell::String(
                                entry
                                    .path()
                                    .strip_prefix(dir)?
                                    .to_string_lossy()
                                    .to_string(),
                            )),
                        );
                    }
                    "mode" => {
                        row.push(
                            "mode",
                            Some(Cell::String(format!("{:04o}", metadata.mode()))),
                        );
                    }
                    _ => {}
                }
            }
            return Ok(Some(()));
        }
    }

    fn end_scan(&mut self) -> Result<()> {
        self.state = None;
        Ok(())
    }

    fn begin_modify(&mut self, options: &HashMap<String, String>) -> Result<()> {
        let Some(path) = options.get("dir") else {
            return Err(DirError::NoOption);
        };

        self.state = Some(
            Update {
                dir: path.to_owned(),
            }
            .into(),
        );

        Ok(())
    }

    fn insert(&mut self, row: &Row) -> Result<()> {
        let Some(State::Update(Update { dir, .. })) = &self.state else {
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

        let file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?;

        if let Some(&Some(Cell::String(mode))) = row.get("mode") {
            let mode = mode_from_str(mode)?;
            fchmod(&file, mode)?;
        }

        Ok(())
    }

    fn update(&mut self, rowid: &Cell, new_row: &Row) -> Result<()> {
        report_info(&format!("{:?} {:?}", rowid, new_row));
        let Some(State::Update(Update { dir, .. })) = &self.state else {
            return Err(DirError::NoOption);
        };

        let Cell::String(path) = rowid else {
            return Err(DirError::NoOption);
        };

        let row = new_row
            .cols
            .iter()
            .cloned()
            .zip(&new_row.cells)
            .collect::<HashMap<_, _>>();

        let dir = Path::new(dir);
        let path = dir.join(path);
        let file = fs::OpenOptions::new()
            .create(false)
            .write(true)
            .open(&path)?;

        if let Some(&Some(Cell::String(mode))) = row.get("mode") {
            let mode = mode_from_str(mode)?;
            fchmod(&file, mode)?;
        }

        // TODO This operation is not supported????
        if let Some(&Some(Cell::String(new_path))) = row.get("path") {
            let new_path = dir.join(new_path);
            fs::rename(path, new_path)?;
        };

        Ok(())
    }

    fn delete(&mut self, rowid: &Cell) -> Result<()> {
        let Some(State::Update(Update { dir, .. })) = &self.state else {
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
        self.state = None;
        Ok(())
    }
}

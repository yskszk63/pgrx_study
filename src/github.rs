use pgrx::PgSqlErrorCode;
use pgrx::pg_sys::panic::ErrorReport;
use supabase_wrappers::prelude::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub path: String,
    pub mode: String,
    pub r#type: String,
    pub sha: String,
    pub size: Option<u64>,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub sha: String,
    pub url: String,
    pub tree: Vec<Item>,
    pub truncated: bool,
}

#[wrappers_fdw(
    version = "0.1.0",
    author = "me",
    error_type = "GithubError",
)]
struct Github {
    response: Option<Box<dyn Iterator<Item=Item>>>,
}

#[derive(Error, Debug)]
enum GithubError {
    #[error("Failed to fetch {0:}")]
    FailedToFetch(#[from] reqwest::Error),

    #[error("Missing options: repo")]
    MissingOpts,

    #[error("Request failure {0:}")]
    RequestFailure(u16),

    #[error("Failed to deserialize {0:}")]
    Deserialize(#[from] serde_json::Error),
}

impl From<GithubError> for ErrorReport {
    fn from(value: GithubError) -> Self {
        ErrorReport::new(PgSqlErrorCode::ERRCODE_FDW_ERROR, value.to_string(), "github")
    }
}

type Result<T> = std::result::Result<T, GithubError>;

impl ForeignDataWrapper<GithubError> for Github {
    fn new(_server: ForeignServer) -> Result<Self>
    where Self: Sized {
        //report_info("Hello, World!");
        Ok(Github{
            response: None,
        })
    }

    fn begin_scan(
        &mut self,
        _quals: &[Qual],
        _columns: &[Column],
        _sorts: &[Sort],
        _limit: &Option<Limit>,
        options: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let repo = options.get("repo");
        let Some(repo) = repo else { return Err(GithubError::MissingOpts); };
        let client = Client::builder().user_agent("My-APP").build()?;

        let res = client
            .get(format!("https://api.github.com/repos/{}/git/trees/main?recursive=1", repo))
            .send()?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            report_warning(&res.text()?);
            return Err(GithubError::RequestFailure(status));
        }

        let res = serde_json::from_reader::<_, Response>(res)?;
        self.response = Some(Box::new(res.tree.into_iter()));

        Ok(())
    }

    fn iter_scan(&mut self, row: &mut Row) -> Result<Option<()>> {
        let Some(ref mut iter) = self.response else {
            return Ok(None);
        };

        let Some (item) = &iter.next() else {
            return Ok(None);
        };

        row.push("path", Some(Cell::String(item.path.clone())));
        row.push("mode", Some(Cell::String(item.mode.clone())));
        row.push("type", Some(Cell::String(item.r#type.clone())));
        row.push("sha", Some(Cell::String(item.sha.clone())));
        if let Some(size) = item.size {
            row.push("size", Some(Cell::I64(size as i64))); // TODO
        } else {
            row.push("size", None);
        };
        row.push("url", Some(Cell::String(item.url.clone())));

        Ok(Some(()))
    }

    fn end_scan(&mut self) -> Result<()> {
        Ok(())
    }
}

use crate::jj::Repo;
use crate::page_writer::PageWriter;
use crate::pages::{self, Page};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn open_split(workspace: &Path) -> Result<PathBuf> {
    let (page_path, page) = {
        let repo = Repo::detect(workspace)?.ok_or_else(|| anyhow!("no jj root found"))?;
        let page_path = repo.workspace_dir().join(".jj/split.jjmagit");

        let mut out = PageWriter::default();
        pages::Split.render(&mut out, &repo)?;

        let page = out.finish().text;
        (page_path, page)
    };

    let mut file = File::create(&page_path).await?;
    file.write_all(page.as_bytes()).await?;

    Ok(page_path)
}

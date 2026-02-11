use crate::jj::Repo;
use crate::page_writer::PageWriter;
use crate::pages::{self, Page};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn open_page(workspace: &Path, page: &dyn Page, arguments: &[&str]) -> Result<PathBuf> {
    let (page_path, page) = {
        let repo = Repo::detect(workspace)?.ok_or_else(|| anyhow!("no jj root found"))?;

        let page_path = pages::path::get_path(repo.workspace_dir(), page, arguments);

        let mut out = PageWriter::default();
        page.render(&mut out, &repo, arguments)?;

        let page = out.finish().text;
        (page_path, page)
    };

    tokio::fs::create_dir_all(&page_path.parent().unwrap()).await?;
    let mut file = File::create(&page_path).await?;
    file.write_all(page.as_bytes()).await?;

    Ok(page_path)
}

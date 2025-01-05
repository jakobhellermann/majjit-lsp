use crate::{
    jj,
    page_writer::PageWriter,
    pages::{self, Page},
};
use anyhow::{ensure, Result};
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn open_split(workspace: &Path) -> Result<PathBuf> {
    let workspace = jj::workspace_root(workspace)?;

    let dot_jj = workspace.join(".jj");
    ensure!(dot_jj.is_dir());

    let page_path = dot_jj.join("split.jjmagit");

    let mut out = PageWriter::default();
    pages::Split.render(&mut out, &workspace)?;

    let mut file = File::create(&page_path).await?;
    file.write_all(out.finish().text.as_bytes()).await?;

    Ok(page_path)
}

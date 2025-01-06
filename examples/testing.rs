#![allow(unused)]
use anyhow::Result;
use jjmagit_language_server::{
    jj::Repo,
    page_writer::PageWriter,
    pages::{self, Page},
};

fn main() -> Result<()> {
    let repo = Repo::detect_cwd()?.unwrap();

    let mut out = PageWriter::default();
    pages::Split.render(&mut out, &repo)?;
    let _page = out.finish();

    let commits = repo.log()?;

    Ok(())
}

#![allow(unused)]
use anyhow::Result;
use jjmagit_language_server::jj::Repo;
use jjmagit_language_server::page_writer::PageWriter;
use jjmagit_language_server::pages::{self, Page};

fn main() -> Result<()> {
    let repo = Repo::detect_cwd()?.unwrap();

    let mut out = PageWriter::default();
    pages::Split.render(&mut out, &repo)?;
    let _page = out.finish();

    let commits = repo.log()?;

    Ok(())
}

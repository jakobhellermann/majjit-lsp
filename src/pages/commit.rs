use crate::jj::Repo;
use crate::page_writer::PageWriter;
use anyhow::{Result, anyhow};
use std::io::Write;

pub struct Commit;

impl super::Page for Commit {
    fn name(&self) -> &'static str {
        "commit"
    }

    fn render(&self, out: &mut PageWriter, _repo: &Repo, options: &[&str]) -> Result<()> {
        let [change_id] = options else {
            return Err(anyhow!("Expected 1 argument change_id, got {:?}", options,));
        };

        writeln!(out, "Commit: {change_id}")?;
        writeln!(out, "todo")?;

        Ok(())
    }
}

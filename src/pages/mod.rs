use crate::jj::Repo;
use crate::page_writer::PageWriter;
use anyhow::Result;

mod status;

pub use status::Status;

pub const ALL: &[&dyn Page] = &[&Status];

pub fn named(name: &str) -> Option<&dyn Page> {
    ALL.iter()
        .find(|page| page.name() == name)
        .map(|page| &**page)
}

pub trait Page: Send + Sync {
    fn name(&self) -> &'static str;

    fn render(&self, out: &mut PageWriter, repo: &Repo) -> Result<()>;
}

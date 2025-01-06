use crate::jj::Repo;
use crate::page_writer::PageWriter;
use anyhow::Result;

mod split;

pub use split::Split;

pub const ALL: &[&dyn Page] = &[&Split];

pub fn named(name: &str) -> Option<&dyn Page> {
    ALL.iter()
        .find(|page| page.name() == name)
        .map(|page| &**page)
}

pub trait Page {
    fn name(&self) -> &'static str;

    fn render(&self, out: &mut PageWriter, repo: &Repo) -> Result<()>;
}

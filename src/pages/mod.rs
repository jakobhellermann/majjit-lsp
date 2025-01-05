use crate::page_writer::PageWriter;
use anyhow::Result;
use std::path::Path;

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

    fn render(&self, out: &mut PageWriter, repo: &Path) -> Result<()>;
}

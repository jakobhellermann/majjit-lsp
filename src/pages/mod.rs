use crate::jj::Repo;
use crate::page_writer::PageWriter;
use anyhow::Result;

mod annotate;
mod status;

pub use annotate::Annotate;
pub use status::Status;

pub const ALL: &[&dyn Page] = &[&Status, &Annotate];

pub fn named(name: &str) -> Option<&'static dyn Page> {
    ALL.iter()
        .find(|page| page.name() == name)
        .map(|page| &**page)
}

pub trait Page: Send + Sync {
    fn name(&self) -> &'static str;

    fn render(&self, out: &mut PageWriter, repo: &Repo, arguments: &[&str]) -> Result<()>;
}

pub mod path {
    use anyhow::{ensure, Context, Result};
    use std::path::{Path, PathBuf};

    use crate::pages::{self, truncate_end_matches};

    use super::Page;

    const _ESCAPE_CHARS: &str = r#"<>:"/\|?*"#;

    fn encode_argument(arg: &str) -> String {
        if r#"<>:"\|?*"#.chars().any(|c| arg.contains(c)) {
            todo!()
        }
        arg.replace('/', "'")
    }
    fn decode_argument(arg: &str) -> String {
        arg.replace('\'', "/")
    }

    pub fn get_path(workspace: &Path, page: &dyn Page, arguments: &[&str]) -> PathBuf {
        let mut page_path = workspace.to_path_buf();
        page_path.push(".jj");

        match arguments {
            [] => page_path.push(format!("{}.jjmagit", page.name())),
            [rest @ .., last] => {
                page_path.push(page.name());
                page_path.extend(rest.iter().copied().map(encode_argument));
                page_path.push(format!("{}.jjmagit", encode_argument(last)));
            }
        }

        page_path
    }

    pub fn parse_path(path: &Path) -> Result<(PathBuf, &'static dyn Page, Vec<String>)> {
        let mut jj_path = PathBuf::new();
        let mut components = path
            .components()
            .map(|c| c.as_os_str().to_str().expect("invalid utf8"));

        let mut found = false;
        for component in components.by_ref() {
            if component == ".jj" {
                found = true;
                break;
            }
            jj_path.push(component);
        }

        let page_name = components
            .next()
            .with_context(|| format!("Path {} does not contain page", path.display()))?
            .trim_end_matches(".jjmagit");

        let page = pages::named(page_name)
            .with_context(|| format!("Page '{}' does not exist", page_name))?;

        ensure!(found, "Path {} not inside .jj directory", path.display());

        let mut arguments = Vec::new();
        for argument in components {
            arguments.push(decode_argument(argument));
        }

        if let Some(argument) = arguments.last_mut() {
            truncate_end_matches(argument, ".jjmagit");
        }

        Ok((jj_path, page, arguments))
    }
}

fn truncate_end_matches(buf: &mut String, suffix: &str) {
    if buf.ends_with(suffix) {
        buf.truncate(buf.len() - suffix.len());
    }
}

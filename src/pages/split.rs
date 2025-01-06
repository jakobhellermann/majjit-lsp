use anyhow::Result;
use jj_lib::copies::CopyOperation;
use jj_lib::matchers::{EverythingMatcher, FilesMatcher};
use std::io::Write;
use tower_lsp::lsp_types::Url;

use crate::jj::Repo;
use crate::page_writer::GotoDefinitionTarget;
use crate::page_writer::PageWriter;

use super::Page;

pub struct Split;

impl Page for Split {
    fn name(&self) -> &'static str {
        "split"
    }

    fn render(&self, out: &mut PageWriter, repo: &Repo) -> Result<()> {
        let commit = repo.current_commit()?;

        let diff_state = repo.diff(&commit)?;
        let diff = diff_state.diff(&EverythingMatcher)?;

        /*let head = jjcli::log(
            repo,
            "@",
            "format_commit_summary_with_refs(self, bookmarks)",
        )?;
        writeln!(out, "Head:     {head}\n")?;*/

        /* let summary = jjcli::log(repo, "@", "self.diff().summary()")?;
        let mut changes: BTreeMap<&str, Vec<&Path>> = BTreeMap::new();

        // CRMAD
        for line in summary.lines() {
            let (sigil, filename) = line.split_once(' ').unwrap();
            changes.entry(sigil).or_default().push(Path::new(filename));
        }*/

        write!(out.labelled(0), "Unselected changes")?;

        writeln!(out, " ({})", 0)?;

        writeln!(out)?;
        write!(out.labelled(0), "Selected changes")?;

        writeln!(out, " ({})", diff.len(),)?;

        for item in diff {
            out.push_fold();
            let (before, after) = item.values?;
            let before_path = item.path.source();
            let after_path = item.path.target();

            if let Some(op) = item.path.copy_operation() {
                let sigil = match op {
                    CopyOperation::Copy => "C",
                    CopyOperation::Rename => "R",
                };
                let path = repo
                    .path_converter()
                    .format_copied_path(before_path, after_path);
                writeln!(out.labelled(3), "{sigil} {path}")?;
            } else {
                let path = repo.path_converter().format_file_path(after_path);

                let base = repo.workspace_dir();

                let target = GotoDefinitionTarget {
                    target: Url::from_file_path(after_path.to_fs_path(base)?).unwrap(),
                };
                out.goto_def.push(&out.buf, target);
                match (before.is_present(), after.is_present()) {
                    (true, true) => writeln!(out.labelled(4), "M {path}")?,
                    (false, true) => writeln!(out.labelled(5), "A {path}")?,
                    (true, false) => writeln!(out.labelled(6), "D {path}")?,
                    (false, false) => unreachable!(),
                }
                out.goto_def.pop(&out.buf);
            }

            let matcher = FilesMatcher::new([item.path.source(), item.path.target()]);
            diff_state.write_diff(&mut out.plaintext(), &matcher)?;
            out.pop_fold();
        }

        writeln!(out)?;
        writeln!(out.labelled(0), "Recent commits")?;

        let log = repo.log()?;
        dbg!(log.len());
        for commit in log {
            out.push_fold();
            repo.write_template(&mut out.formatter(), &commit)?;
            writeln!(out)?;

            let diff = repo.diff(&commit)?;
            diff.write_summary(&mut out.formatter())?;
            out.pop_fold();
        }

        Ok(())
    }
}

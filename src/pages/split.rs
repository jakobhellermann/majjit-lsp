use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use tower_lsp::lsp_types::Url;

use crate::jj;
use crate::page_writer::{GotoDefinitionTarget, PageWriter};

use super::Page;

pub struct Split;

impl Page for Split {
    fn name(&self) -> &'static str {
        "split"
    }

    fn render(&self, out: &mut PageWriter, repo: &Path) -> Result<()> {
        let head = jj::log(
            repo,
            "@",
            "format_commit_summary_with_refs(self, bookmarks)",
        )?;
        writeln!(out, "Head:     {head}\n")?;

        let summary = jj::log(repo, "@", "self.diff().summary()")?;
        let mut changes: BTreeMap<&str, Vec<&Path>> = BTreeMap::new();

        // CRMAD
        for line in summary.lines() {
            let (sigil, filename) = line.split_once(' ').unwrap();
            changes.entry(sigil).or_default().push(Path::new(filename));
        }

        write!(out.labelled(0), "Unselected changes")?;

        writeln!(out, " ({})", 0)?;

        writeln!(out)?;
        write!(out.labelled(0), "Selected changes")?;
        writeln!(
            out,
            " ({})",
            changes.values().map(|x| x.len()).sum::<usize>()
        )?;
        for (sigil, files) in changes {
            for filename in files {
                write!(out.labelled(2), "{}", sigil)?;
                write!(out, "\t")?;

                let path = repo.join(filename);
                let target = GotoDefinitionTarget {
                    target: Url::from_file_path(&path)
                        .map_err(|_| anyhow!("couldn't turn path {} into url", path.display()))?,
                };
                out.push_fold();
                writeln!(out.goto_def(target), "{}", filename.display())?;
                let diff = jj::diff(repo, filename)?;
                writeln!(out, "{}", diff.trim())?;
                out.pop_fold();
            }
        }

        writeln!(out, "")?;

        writeln!(out.labelled(0), "Recent commits")?;

        let recent_commits = jj::log(
            repo,
            "present(@) | ancestors(immutable_heads().., 4) | present(trunk())",
            "builtin_log_oneline",
        )?;
        for recent_commit in recent_commits.lines() {
            out.push_fold();

            // HACK
            let commit_id = recent_commit.split_once(" ").unwrap().0;
            let commit_info = jj::show(repo, commit_id)?;

            write!(out, "{}", recent_commit)?;
            write!(out, "{}", commit_info)?;
            out.pop_fold();
        }

        Ok(())
    }
}

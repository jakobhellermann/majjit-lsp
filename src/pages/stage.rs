use anyhow::Result;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::jj;
use crate::page_writer::PageWriter;

pub fn render(out: &mut PageWriter, repo: &Path) -> Result<()> {
    let head = jj::log(
        repo,
        "@",
        "format_commit_summary_with_refs(self, bookmarks)",
    )?;
    writeln!(out, "Head:     {head}\n")?;

    let summary = jj::log(repo, "@", "self.diff().summary()")?;
    let mut changes: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    // CRMAD
    for line in summary.lines() {
        let (sigil, filename) = line.split_once(' ').unwrap();
        changes.entry(sigil).or_default().push(filename);
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
            writeln!(out, "{}\t{}", sigil, filename)?;
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

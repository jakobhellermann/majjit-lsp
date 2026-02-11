use anyhow::Result;
use jj_lib::copies::CopyOperation;
use jj_lib::matchers::{EverythingMatcher, FilesMatcher};
use std::io::Write;
use tower_lsp::lsp_types::Url;

use crate::jj::Repo;
use crate::page_writer::{CodeAction, GotoDefinitionTarget, PageWriter};
use crate::semantic_token;

use super::Page;

pub struct Status;

impl Page for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn render(&self, out: &mut PageWriter, repo: &Repo, _: &[&str]) -> Result<()> {
        let commit = repo.current_commit()?;

        let diff_state = repo.diff(&commit)?;
        let diff = diff_state.diff(&EverythingMatcher)?;

        write!(out.labelled(0), "Head: ")?;

        repo.write_log(&mut out.formatter(), &commit)?;
        writeln!(out)?;

        out.push_code_action(CodeAction::move_to_commit());
        write!(out.labelled(0), "Changes")?;
        out.pop_code_action();

        writeln!(out, " ({})", diff.len(),)?;

        for item in diff {
            let (before, after) = item.values?;
            let before_path = item.path.source();
            let after_path = item.path.target();

            let pretty_path = match item.path.copy_operation() {
                Some(_) => repo
                    .path_converter()
                    .format_copied_path(before_path, after_path),
                None => repo.path_converter().format_file_path(after_path),
            };

            out.push_fold();
            out.push_code_action(CodeAction::move_file_to_commit(pretty_path.clone()));

            if let Some(op) = item.path.copy_operation() {
                let (label, sigil) = match op {
                    CopyOperation::Copy => ("created", "C"),
                    CopyOperation::Rename => ("renamed", "R"),
                };
                writeln!(
                    out.labelled(semantic_token::get(label)),
                    "{sigil} {pretty_path}"
                )?;
            } else {
                let path = repo.path_converter().format_file_path(after_path);

                let base = repo.workspace_dir();

                let target = GotoDefinitionTarget {
                    target: Url::from_file_path(after_path.to_fs_path(base)?).unwrap(),
                };
                out.goto_def.push(&out.buf, target);
                match (before.is_present(), after.is_present()) {
                    (true, true) => {
                        let label = semantic_token::get("modified");
                        writeln!(out.labelled(label), "M {path}")?
                    }
                    (false, true) => {
                        let label = semantic_token::get("added");
                        writeln!(out.labelled(label), "A {path}")?
                    }
                    (true, false) => {
                        let label = semantic_token::get("deleted");
                        writeln!(out.labelled(label), "D {path}")?
                    }
                    (false, false) => unreachable!(),
                }
                out.goto_def.pop(&out.buf);
            }

            let matcher = FilesMatcher::new([item.path.source(), item.path.target()]);
            diff_state.write_diff(&mut out.formatter(), &matcher)?;
            out.pop_fold();
            out.pop_code_action();
        }

        writeln!(out)?;
        writeln!(out.labelled(0), "Recent commits")?;

        let log = repo.log()?;
        for commit in log {
            out.push_fold();

            out.push_code_actions(vec![CodeAction::new(&commit), CodeAction::abandon(&commit)]);
            repo.write_log(&mut out.formatter(), &commit)?;
            out.pop_code_action();
            // writeln!(out)?;

            let diff = repo.diff(&commit)?;
            diff.write_summary(&mut out.formatter())?;

            out.pop_fold();
        }

        Ok(())
    }
}

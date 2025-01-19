use anyhow::{anyhow, Result};
use jj_cli::templater::TemplateRenderer;
use jj_lib::annotate::FileAnnotation;
use jj_lib::commit::Commit;
use std::io::Write;

use crate::jj::Repo;
use crate::page_writer::{CodeAction, PageWriter};

use super::Page;

pub struct Annotate;

impl Page for Annotate {
    fn name(&self) -> &'static str {
        "annotate"
    }

    fn render(&self, out: &mut PageWriter, repo: &Repo, options: &[&str]) -> Result<()> {
        let [file_path] = options else {
            return Err(anyhow!("Expected 1 argument filename, got {:?}", options,));
        };

        let starting_commit = repo.revset_single("@")?;
        let template = repo.settings_commit_template("templates.annotate_commit_summary")?;
        let annotation = repo.annotation(&starting_commit, file_path)?;

        render_file_annotation(repo.inner(), out, &template, &annotation)?;

        Ok(())
    }
}

fn render_file_annotation(
    repo: &dyn jj_lib::repo::Repo,
    out: &mut PageWriter,
    template_render: &TemplateRenderer<Commit>,
    annotation: &FileAnnotation,
) -> Result<()> {
    for (line_no, (commit_id, line)) in annotation.lines().enumerate() {
        out.push_code_action(CodeAction::annotate_before());
        let commit_id = commit_id.expect("should reached to the empty ancestor");

        let commit = repo.store().get_commit(commit_id)?;
        template_render.format(&commit, &mut out.formatter())?;
        write!(out, " {:>4}: ", line_no + 1)?;
        out.write_all(line)?;

        out.pop_code_action();
    }

    Ok(())
}

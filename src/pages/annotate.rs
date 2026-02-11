use anyhow::{Result, anyhow};
use jj_cli::commit_templater::AnnotationLine;
use jj_cli::templater::TemplateRenderer;
use jj_lib::annotate::{FileAnnotation, LineOrigin};

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
        let template = repo.settings_annotation_template("templates.file_annotate")?;
        let annotation = repo.annotation(&starting_commit, file_path)?;

        render_file_annotation(repo.inner(), out, &template, &annotation)?;

        Ok(())
    }
}

fn render_file_annotation(
    repo: &dyn jj_lib::repo::Repo,
    out: &mut PageWriter,
    template_render: &TemplateRenderer<AnnotationLine>,
    annotation: &FileAnnotation,
) -> Result<()> {
    let mut last_id = None;
    let default_line_origin = LineOrigin {
        commit_id: repo.store().root_commit_id().clone(),
        line_number: 0,
    };
    for (line_number, (line_origin, content)) in annotation.line_origins().enumerate() {
        out.push_code_action(CodeAction::annotate_before());

        let line_origin = line_origin.unwrap_or(&default_line_origin);
        let commit = repo.store().get_commit(&line_origin.commit_id)?;
        let first_line_in_hunk = last_id != Some(&line_origin.commit_id);
        let annotation_line = AnnotationLine {
            commit,
            content: content.to_owned(),
            line_number: line_number + 1,
            original_line_number: line_origin.line_number + 1,
            first_line_in_hunk,
        };
        last_id = Some(&line_origin.commit_id);

        template_render.format(&annotation_line, &mut out.formatter())?;

        out.pop_code_action();
    }

    Ok(())
}

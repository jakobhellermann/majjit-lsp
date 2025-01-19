use jj_cli::formatter::{Formatter, PlainTextFormatter};
use jj_lib::commit::Commit;
use tower_lsp::lsp_types::Url;

use crate::semantic_token;
use crate::span::Span;
use std::fmt::Write as _;

type TokenType = u32;

#[derive(Debug)]
pub struct Page {
    pub text: String,
    pub labels: Vec<(Span, TokenType)>,
    pub folding_ranges: Vec<(Span, ())>,
    pub goto_def: Vec<(Span, GotoDefinitionTarget)>,
    pub code_actions: Vec<(Span, CodeAction)>,
}

#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub command: &'static str,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GotoDefinitionTarget {
    pub target: Url,
}

#[derive(Default)]
pub struct PageWriter {
    pub buf: String,

    pub labels: WriterStack<TokenType, true>,
    pub folds: WriterStack<()>,
    pub goto_def: WriterStack<GotoDefinitionTarget>,
    pub code_actions: WriterStack<Vec<CodeAction>>,

    pub debug: bool,
}

pub struct WriterStack<T, const DISJOINT: bool = false> {
    in_progress: Vec<(usize, T)>,
    done: Vec<(Span, T)>,
}

impl PageWriter {
    pub fn finish(mut self) -> Page {
        self.labels.done.sort_by_key(|(range, _)| range.start);

        Page {
            text: self.buf,
            labels: self.labels.done,
            folding_ranges: self.folds.done,
            goto_def: self.goto_def.done,
            code_actions: self
                .code_actions
                .done
                .into_iter()
                .flat_map(|(range, item)| (item.into_iter().map(move |item| (range.clone(), item))))
                .collect(),
        }
    }

    pub fn labelled(&mut self, token_type: TokenType) -> ScopedWriter<'_, TokenType, true> {
        ScopedWriter {
            buf: &mut self.buf,
            stack: &mut self.labels,
            data: Some(token_type),
        }
    }
    pub fn folding(&mut self) -> ScopedWriter<'_, ()> {
        ScopedWriter {
            buf: &mut self.buf,
            stack: &mut self.folds,
            data: Some(()),
        }
    }
    pub fn goto_def(
        &mut self,
        target: GotoDefinitionTarget,
    ) -> ScopedWriter<'_, GotoDefinitionTarget> {
        ScopedWriter {
            buf: &mut self.buf,
            stack: &mut self.goto_def,
            data: Some(target),
        }
    }

    pub fn push_fold(&mut self) {
        self.folds.push(&self.buf, ());
    }
    pub fn pop_fold(&mut self) {
        self.folds.pop(&self.buf);
    }

    pub fn push_code_action(&mut self, code_action: CodeAction) {
        self.code_actions.push(&self.buf, vec![code_action]);
    }
    pub fn push_code_actions(&mut self, code_actions: Vec<CodeAction>) {
        self.code_actions.push(&self.buf, code_actions);
    }
    pub fn pop_code_action(&mut self) {
        self.code_actions.pop(&self.buf);
    }

    pub fn plaintext(&mut self) -> impl Formatter + '_ {
        PlainTextFormatter::new(&mut *self)
    }

    pub fn formatter(&mut self) -> FormatterAdapter<'_> {
        FormatterAdapter {
            debug: self.debug,
            writer: self,
        }
    }
}

pub struct FormatterAdapter<'a> {
    writer: &'a mut PageWriter,
    debug: bool,
}
impl FormatterAdapter<'_> {
    pub fn debug(self) -> Self {
        FormatterAdapter {
            writer: self.writer,
            debug: true,
        }
    }
}
impl std::io::Write for FormatterAdapter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
impl Formatter for FormatterAdapter<'_> {
    fn raw(&mut self) -> std::io::Result<Box<dyn std::io::Write + '_>> {
        Ok(Box::new(&mut self.writer))
    }

    fn push_label(&mut self, label: &str) -> std::io::Result<()> {
        if self.debug {
            self.writer.buf.push_str(label);
            self.writer.buf.push('(');
        }

        let token = semantic_token::get_or_default(label);

        self.writer.labels.push(&self.writer.buf, token);
        Ok(())
    }

    fn pop_label(&mut self) -> std::io::Result<()> {
        if self.debug {
            let has_newline = self.writer.buf.strip_suffix("\n").is_some();
            if has_newline {
                self.writer.buf.truncate(self.writer.buf.len() - 1);
            }
            self.writer.buf.push(')');
            if has_newline {
                self.writer.buf.push('\n');
            }
        }
        self.writer.labels.pop(&self.writer.buf);
        Ok(())
    }
}

impl std::io::Write for PageWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let str = std::str::from_utf8(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.buf.push_str(str);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl<T, const DISJOINT: bool> Default for WriterStack<T, DISJOINT> {
    fn default() -> Self {
        Self {
            in_progress: Default::default(),
            done: Default::default(),
        }
    }
}
impl<T: Clone, const DISJOINT: bool> WriterStack<T, DISJOINT> {
    pub fn push(&mut self, buf: &str, data: T) {
        let offset = buf.len();

        if DISJOINT {
            if let Some((top_start, top_data)) = self.in_progress.last() {
                self.done.push((*top_start..offset, top_data.clone()));
            }
        }

        self.in_progress.push((offset, data));
    }

    pub fn pop(&mut self, buf: &str) {
        let offset_end = buf.trim_end().len();
        let (offset_start, data) = self.in_progress.pop().expect("pop without push");

        if let Some((start, _)) = self.in_progress.last_mut() {
            *start = offset_end;
        }

        self.done.push((offset_start..offset_end, data));
    }
}

pub struct ScopedWriter<'a, T, const DISJOINT: bool = false> {
    stack: &'a mut WriterStack<T, DISJOINT>,
    buf: &'a mut String,
    data: Option<T>,
}

impl<T: Clone, const DISJOINT: bool> ScopedWriter<'_, T, DISJOINT> {
    pub fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.stack.push(
            self.buf,
            self.data
                .take()
                .expect("multiple write_fmt on ScopedWriter"),
        );
        let _ = self.buf.write_fmt(fmt);
        self.stack.pop(self.buf);
        Ok(())
    }
}

impl CodeAction {
    pub fn move_to_commit() -> Self {
        CodeAction {
            title: "Move to first commit".into(),
            command: "todo",
            args: vec![],
        }
    }

    pub fn move_file_to_commit(pretty_path: String) -> Self {
        CodeAction {
            title: format!("Move {} to first commit", pretty_path),
            command: "todo",
            args: vec![pretty_path],
        }
    }

    pub fn abandon(commit: &Commit) -> Self {
        CodeAction {
            title: format!("Abandon commit {}", commit.change_id()),
            command: "todo",
            args: vec![commit.change_id().to_string()],
        }
    }

    pub fn new(commit: &Commit) -> Self {
        CodeAction {
            title: format!("Create new commit at {}", commit.change_id()),
            command: "todo",
            args: vec![commit.change_id().to_string()],
        }
    }

    pub fn annotate_before() -> Self {
        CodeAction {
            title: "Annotate before this commit".into(),
            command: "todo",
            args: vec![],
        }
    }
}

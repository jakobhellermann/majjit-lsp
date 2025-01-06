use jj_cli::formatter::{Formatter, PlainTextFormatter};
use tower_lsp::lsp_types::Url;

use crate::{semantic_token::LEGEND_TYPE, span::Span};
use std::fmt::Write as _;

type TokenType = usize;

#[derive(Debug)]
pub struct Page {
    pub text: String,
    pub labels: Vec<(Span, TokenType)>,
    pub folding_ranges: Vec<(Span, ())>,
    pub goto_def: Vec<(Span, GotoDefinitionTarget)>,
}

#[derive(Debug)]
pub struct GotoDefinitionTarget {
    pub target: Url,
}

#[derive(Default)]
pub struct PageWriter {
    pub buf: String,

    pub labels: WriterStack<TokenType>,
    pub folds: WriterStack<()>,
    pub goto_def: WriterStack<GotoDefinitionTarget>,
}

pub struct WriterStack<T> {
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
        }
    }
    pub fn labelled(&mut self, token_type: TokenType) -> ScopedWriter<'_, usize> {
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

    pub fn plaintext(&mut self) -> impl Formatter + '_ {
        PlainTextFormatter::new(&mut *self)
    }

    pub fn formatter(&mut self) -> impl Formatter + '_ {
        FormatterAdapter {
            writer: self,
            debug: false,
        }
    }
}

pub struct FormatterAdapter<'a> {
    writer: &'a mut PageWriter,
    debug: bool,
}
impl<'a> std::io::Write for FormatterAdapter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
impl<'a> Formatter for FormatterAdapter<'a> {
    fn raw(&mut self) -> std::io::Result<Box<dyn std::io::Write + '_>> {
        Ok(Box::new(&mut self.writer))
    }

    fn push_label(&mut self, label: &str) -> std::io::Result<()> {
        if self.debug {
            self.writer.buf.push_str(label);
            self.writer.buf.push_str("(");
        }

        let idx = (label.chars().next().unwrap() as u8) as usize % LEGEND_TYPE.len();

        self.writer.labels.push(&self.writer.buf, idx);
        Ok(())
    }

    fn pop_label(&mut self) -> std::io::Result<()> {
        if self.debug {
            self.writer.buf.push_str(")");
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

impl<T> Default for WriterStack<T> {
    fn default() -> Self {
        Self {
            in_progress: Default::default(),
            done: Default::default(),
        }
    }
}
impl<T> WriterStack<T> {
    pub fn push(&mut self, buf: &str, data: T) {
        let offset = buf.len();
        self.in_progress.push((offset, data));
    }
    pub fn pop(&mut self, buf: &str) {
        let offset_end = buf.trim_end().len();
        let (offset_start, data) = self.in_progress.pop().expect("pop without push");
        self.done.push((offset_start..offset_end, data));
    }
}

pub struct ScopedWriter<'a, T> {
    stack: &'a mut WriterStack<T>,
    buf: &'a mut String,
    data: Option<T>,
}

impl<T> ScopedWriter<'_, T> {
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

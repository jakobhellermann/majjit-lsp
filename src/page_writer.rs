use crate::span::Span;
use std::fmt::Write as _;

type TokenType = usize;

#[derive(Debug)]
pub struct Page {
    pub text: String,
    pub labels: Vec<(Span, TokenType)>,
    pub folding_ranges: Vec<(Span, ())>,
}

#[derive(Default)]
pub struct PageWriter {
    buf: String,

    labels: WriterStack<usize>,
    folds: WriterStack<()>,
}

struct WriterStack<T> {
    in_progress: Vec<(usize, T)>,
    done: Vec<(Span, T)>,
}

impl PageWriter {
    pub fn finish(self) -> Page {
        Page {
            text: self.buf,
            labels: self.labels.done,
            folding_ranges: self.folds.done,
        }
    }
    pub fn labelled(&mut self, token_type: TokenType) -> ScopedWriter<'_, usize> {
        ScopedWriter {
            buf: &mut self.buf,
            stack: &mut self.labels,
            data: token_type,
        }
    }
    pub fn folding(&mut self) -> ScopedWriter<'_, ()> {
        ScopedWriter {
            buf: &mut self.buf,
            stack: &mut self.folds,
            data: (),
        }
    }

    pub fn push_fold(&mut self) {
        self.folds.push(&self.buf, ());
    }
    pub fn pop_fold(&mut self) {
        self.folds.pop(&self.buf);
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
    data: T,
}

impl<'a, T: Copy> ScopedWriter<'a, T> {
    pub fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.stack.push(&self.buf, self.data);
        let _ = self.buf.write_fmt(fmt);
        self.stack.pop(&self.buf);
        Ok(())
    }
}

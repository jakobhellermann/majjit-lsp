#![allow(unused)]
use anyhow::Result;
use jjmagit_language_server::jj::Repo;
use jjmagit_language_server::page_writer::PageWriter;
use jjmagit_language_server::pages::{self, Page};
use std::io::Write;

fn main() -> Result<()> {
    let repo = Repo::detect_cwd()?.unwrap();

    let mut out = PageWriter::default();

    out.labels.push(&out.buf, 0);
    write!(out, "aaa");
    out.labels.push(&out.buf, 1);
    write!(out, "bbb");
    out.labels.pop(&out.buf);
    write!(out, "aaa");
    out.labels.pop(&out.buf);

    let page = out.finish();
    println!("{}", page.text);

    Ok(())
}

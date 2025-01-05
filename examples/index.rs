use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;

#[derive(Default)]
struct Formatter {
    buf: String,
}
impl std::io::Write for Formatter {
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

fn jj_command<'c>(command: &'c mut Command, repo: &str) -> &'c mut Command {
    command
        .env("JJ_CONFIG", "/dev/null")
        .args(&["--repository", repo])
}

fn log(repo: &str, revisions: &str, template: &str) -> Result<String> {
    let output = jj_command(&mut Command::new("jj"), repo)
        .args(&["log", "--no-graph", "-r", revisions, "-T", template])
        .output()?;
    anyhow::ensure!(output.status.success(), String::from_utf8(output.stderr)?);

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

fn split_page(out: &mut impl std::io::Write, repo: &str) -> Result<()> {
    let head = log(
        repo,
        "@",
        "format_commit_summary_with_refs(self, bookmarks)",
    )?;
    writeln!(out, "Head:     {head}\n")?;

    let summary = log(repo, "@", "self.diff().summary()")?;
    let mut changes: HashMap<&str, Vec<&str>> = HashMap::with_capacity(4);

    // CRMAD
    for line in summary.lines() {
        let (sigil, filename) = line.split_once(' ').unwrap();
        changes.entry(sigil).or_default().push(filename);
    }

    writeln!(out, "Unselected changes (0)")?;

    writeln!(
        out,
        "\nSelected changes ({})",
        changes.values().map(|x| x.len()).sum::<usize>()
    )?;
    for (sigil, files) in changes {
        for filename in files {
            writeln!(out, "{}\t{}", sigil, filename)?;
        }
    }

    writeln!(out, "\nRecent commits")?;

    let summary = log(
        repo,
        "present(@) | ancestors(immutable_heads().., 4) | present(trunk())",
        "builtin_log_oneline",
    )?;
    write!(out, "{}", summary)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let repo = "/home/jakob/dev/jj/jj";

    let mut out = Formatter::default();
    split_page(&mut out, repo)?;
    print!("{}", out.buf);

    Ok(())
}

/*
Unstaged changes (1)
modified   a.js

Staged changes (1)
Recent commits
977dfbb main initial commit


*/

use log::warn;
use tower_lsp::lsp_types::SemanticTokenType;

macro_rules! semantic_tokens {
    ($($preset:ident,)* $($name:literal),* $(,)?) => {
        pub const LEGEND_TYPE: &[SemanticTokenType] =
            &[
                $(
                    SemanticTokenType::$preset,
                )*
                $(
                    SemanticTokenType::new($name),
                )*
            ];


    };
}

#[track_caller]
pub fn get(name: &'static str) -> u32 {
    try_get(name).unwrap()
}
pub fn try_get(name: &str) -> Option<u32> {
    LEGEND_TYPE
        .iter()
        .position(|x| x.as_str() == name)
        .map(|x| x as u32)
}
pub fn get_or_default(name: &str) -> u32 {
    try_get(name).unwrap_or_else(|| {
        warn!("unknown semantic scope: {name}");
        0
    })
}

semantic_tokens!(
    FUNCTION,
    VARIABLE,
    "access-denied",
    "added",
    "author",
    "bar",
    "binary",
    "bookmark_list",
    "bookmark",
    "bookmarks",
    "branch",
    "branches",
    "change_id",
    "commit_id",
    "committer",
    "config_list",
    "conflict_description",
    "conflict",
    "copied",
    "current_operation",
    "difficult",
    "divergent",
    "domain",
    "elided",
    "empty",
    "error_source",
    "error",
    "file_header",
    "foo",
    "git_head",
    "git_refs",
    "header",
    "heading",
    "hint",
    "hunk_header",
    "id",
    "immutable",
    "inner",
    "line_number",
    "local_bookmarks",
    "local_branches",
    "local",
    "log",
    "modified",
    "name",
    "node",
    "op_log",
    "operation",
    "overridden",
    "placeholder",
    "prefix",
    "remote_bookmarks",
    "remote_branches",
    "remote",
    "removed",
    "renamed",
    "rest",
    "root",
    "separator",
    "stat-summary",
    "tag_list",
    "tag",
    "tags",
    "time",
    "timestamp",
    "token",
    "user",
    "value",
    "warning",
    "working_copies",
    "working_copy",
    //
    "ago",
    "context",
    "description",
    "diff",
    "first_line",
    "shortest",
    "summary",
);

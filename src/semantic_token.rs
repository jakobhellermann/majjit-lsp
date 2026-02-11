use log::trace;
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
    try_get(name)
        .ok_or_else(|| format!("unknown token: {name}"))
        .unwrap()
}
pub fn try_get(name: &str) -> Option<u32> {
    LEGEND_TYPE
        .iter()
        .position(|x| x.as_str() == name)
        .map(|x| x as u32)
}
pub fn get_or_default(name: &str) -> u32 {
    try_get(name).unwrap_or_else(|| {
        trace!("unknown semantic scope: {name}");
        0
    })
}

semantic_tokens!(
    // jjmagit labels
    "jjmagit",
    // jj labels
    "access-denied",
    "added",
    "author",
    "bad",
    "binary",
    "bookmark",
    "bookmarks",
    "change_id",
    "change_offset",
    "commit_id",
    "committer",
    "config_list",
    "conflict",
    "conflict_description",
    "conflicted",
    "context",
    "copied",
    "current_operation",
    "description",
    "diff",
    "difficult",
    "display",
    "divergent",
    "elided",
    "empty",
    "error",
    "error_source",
    "file_header",
    "git_head",
    "git_ref",
    "git_refs",
    "good",
    "header",
    "heading",
    "hidden",
    "hint",
    "hunk_header",
    "id",
    "immutable",
    "invalid",
    "key",
    "line_number",
    "local_bookmarks",
    "modified",
    "mutable",
    "name",
    "node",
    "operation",
    "overridden",
    "path",
    "placeholder",
    "prefix",
    "remote_bookmarks",
    "removed",
    "renamed",
    "rest",
    "root",
    "separator",
    "signature",
    "snapshot",
    "source",
    "status",
    "tag",
    "tags",
    "time",
    "timestamp",
    "token",
    "unknown",
    "untracked",
    "user",
    "value",
    "warning",
    "working_copies",
    "working_copy",
    //
    "created",
    "renamed",
    "modified",
    "added",
    "deleted",
);

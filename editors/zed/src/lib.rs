use zed_extension_api::{self as zed, Result};

struct JJMagit {}

impl zed::Extension for JJMagit {
    fn new() -> Self {
        JJMagit {}
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<zed_extension_api::Command> {
        let command = worktree
            .which("jjmagit-language-server")
            .ok_or_else("Couldn't find jjmagit-language-server binary")?;
        Ok(zed_extension_api::Command {
            command,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(JJMagit);

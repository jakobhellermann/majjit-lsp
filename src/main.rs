#![allow(clippy::redundant_closure_call)]
use std::path::Path;

use anyhow::{Context, anyhow};
use dashmap::DashMap;
use jjmagit_language_server::jj::Repo;
use jjmagit_language_server::page_writer::{Page, PageWriter};
use jjmagit_language_server::pages::{self};
use jjmagit_language_server::semantic_token::LEGEND_TYPE;
use log::{debug, trace};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
    page_map: DashMap<String, Page>,

    workspace_folders: RwLock<Vec<Url>>,
}

mod commands {
    pub const OPEN: &str = "open";
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                inlay_hint_provider: None,
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                completion_provider: None,
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![commands::OPEN.to_string()],
                    work_done_progress_options: Default::default(),
                }),

                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("jjmagit".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(true),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        debug!("initialized!");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("file opened: {}", params.text_document.uri);

        let item = TextDocumentItem {
            uri: params.text_document.uri,
            text: &params.text_document.text,
            version: Some(params.text_document.version),
        };
        if let Err(e) = self.on_change(item).await {
            log::error!("Error during did_open: {}", e);
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        debug!("file changed: {}", params.text_document.uri);

        let item = TextDocumentItem {
            text: &params.content_changes[0].text,
            uri: params.text_document.uri,
            version: Some(params.text_document.version),
        };
        if let Err(e) = self.on_change(item).await {
            log::error!("Error during did_change: {}", e);
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("file saved!");
        if let Some(text) = params.text {
            let item = TextDocumentItem {
                uri: params.text_document.uri,
                text: &text,
                version: None,
            };
            if let Err(e) = self.on_change(item).await {
                log::error!("Error during did_save: {}", e);
            }

            _ = self.client.semantic_tokens_refresh().await;
        }
    }
    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        debug!("file closed!");
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        debug!("goto defintion");
        let definition = || -> Option<GotoDefinitionResponse> {
            let uri = params.text_document_position_params.text_document.uri;
            let page = self.page_map.get(uri.as_str())?;
            let rope = self.document_map.get(uri.as_str())?;
            let position = params.text_document_position_params.position;
            let offset = position_to_offset(position, &rope)?;

            let goto_def = page
                .goto_def
                .iter()
                .rfind(|(span, _)| span.contains(&offset));

            goto_def.and_then(|(range, target)| {
                let start_position = offset_to_position(range.start, &rope)?;
                let end_position = offset_to_position(range.end, &rope)?;

                let target_range = Range::default();
                Some(GotoDefinitionResponse::Link(vec![LocationLink {
                    origin_selection_range: Some(Range {
                        start: start_position,
                        end: end_position,
                    }),
                    target_uri: target.target.clone(),
                    target_range,
                    target_selection_range: target_range,
                }]))
            })
        }();
        Ok(definition)
    }

    async fn references(&self, _params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        debug!("goto references");
        /*let reference_list = || -> Option<Vec<Location>> {
            let uri = params.text_document_position.text_document.uri;
            let semantic = self.semantic_map.get(uri.as_str())?;
            let rope = self.document_map.get(uri.as_str())?;
            let position = params.text_document_position.position;
            let offset = position_to_offset(position, &rope)?;
            let reference_span_list = get_references(&semantic, offset, offset + 1, false)?;

            let ret = reference_span_list
                .into_iter()
                .filter_map(|range| {
                    let start_position = offset_to_position(range.start, &rope)?;
                    let end_position = offset_to_position(range.end, &rope)?;

                    let range = Range::new(start_position, end_position);

                    Some(Location::new(uri.clone(), range))
                })
                .collect::<Vec<_>>();
            Some(ret)
        }();
        Ok(reference_list)*/
        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();
        trace!("semantic_token_full");
        let semantic_tokens = || -> Option<Vec<SemanticToken>> {
            let page = &mut self.page_map.get_mut(&uri)?;
            let im_complete_tokens = &mut page.labels;
            let rope = self.document_map.get(&uri)?;
            im_complete_tokens.sort_by_key(|a| a.0.start);
            let mut pre_line = 0;
            let mut pre_start = 0;
            let semantic_tokens = im_complete_tokens
                .iter()
                .filter_map(|&(ref range, token_type)| {
                    let line = rope.try_byte_to_line(range.start).ok()? as u32;
                    let first = rope.try_line_to_char(line as usize).ok()? as u32;
                    let start = rope.try_byte_to_char(range.start).ok()? as u32 - first;
                    let delta_line = line - pre_line;
                    let delta_start = if delta_line == 0 {
                        start - pre_start
                    } else {
                        start
                    };
                    let ret = SemanticToken {
                        delta_line,
                        delta_start,
                        length: range.len() as u32,
                        token_type,
                        token_modifiers_bitset: 0,
                    };
                    pre_line = line;
                    pre_start = start;
                    Some(ret)
                })
                .collect::<Vec<_>>();
            Some(semantic_tokens)
        }();
        if let Some(semantic_token) = semantic_tokens {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_token,
            })));
        }
        Ok(None)
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        trace!("semantic_token_range");

        let uri = params.text_document.uri.to_string();
        let semantic_tokens = || -> Option<Vec<SemanticToken>> {
            let page = self.page_map.get(&uri)?;
            let rope = self.document_map.get(&uri)?;
            let mut prev_line = 0;
            let mut prev_start = 0;
            let semantic_tokens = page
                .labels
                .iter()
                .filter_map(|token| {
                    let line = rope.try_byte_to_line(token.0.start).ok()? as u32;
                    let first = rope.try_line_to_char(line as usize).ok()? as u32;
                    let start = rope.try_byte_to_char(token.0.start).ok()? as u32 - first;
                    let ret = SemanticToken {
                        delta_line: line - prev_line,
                        delta_start: if start >= prev_start {
                            start - prev_start
                        } else {
                            start
                        },
                        length: token.0.len() as u32,
                        token_type: token.1,
                        token_modifiers_bitset: 0,
                    };
                    prev_line = line;
                    prev_start = start;
                    Some(ret)
                })
                .collect::<Vec<_>>();
            Some(semantic_tokens)
        }();

        Ok(semantic_tokens.map(|data| {
            SemanticTokensRangeResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })
        }))
    }

    async fn folding_range(
        &self,
        params: tower_lsp::lsp_types::FoldingRangeParams,
    ) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri.to_string();
        trace!("folding ranges");

        let folding_ranges = || -> Option<Vec<FoldingRange>> {
            let page = self.page_map.get(&uri)?;
            let rope = self.document_map.get(&uri)?;
            let folding_ranges = page
                .folding_ranges
                .iter()
                .filter_map(|(range, _)| {
                    let start_line = rope.try_byte_to_line(range.start).ok()? as u32;
                    let end_line = rope.try_byte_to_line(range.end).ok()? as u32;
                    Some(FoldingRange {
                        start_line,
                        start_character: None,
                        end_line,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    })
                })
                .collect::<Vec<_>>();
            Some(folding_ranges)
        }();

        Ok(folding_ranges)
    }

    async fn code_action(
        &self,
        params: tower_lsp::lsp_types::CodeActionParams,
    ) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let code_actions = || -> Option<Vec<CodeActionOrCommand>> {
            let uri = params.text_document.uri;
            let page = self.page_map.get(uri.as_str())?;
            let rope = self.document_map.get(uri.as_str())?;
            let action_range = range_to_offset(params.range, &rope)?;

            let code_actions = page
                .code_actions
                .iter()
                .filter(|(range, _)| intersects(&action_range, range))
                .map(|(_range, action)| {
                    CodeActionOrCommand::Command(Command {
                        title: action.title.clone(),
                        command: action.command.to_owned(),
                        arguments: Some(
                            action
                                .args
                                .iter()
                                .map(|x| Value::String(x.to_owned()))
                                .collect(),
                        ),
                    })
                })
                .collect::<Vec<_>>();
            Some(code_actions)
        }();

        Ok(code_actions)
    }

    async fn inlay_hint(
        &self,
        _params: tower_lsp::lsp_types::InlayHintParams,
    ) -> Result<Option<Vec<InlayHint>>> {
        debug!("inlay hint");
        /*let uri = &params.text_document.uri;
        let mut hashmap = HashMap::new();
        if let Some(ast) = self.ast_map.get(uri.as_str()) {
            ast.iter().for_each(|(func, _)| {
                type_inference(&func.body, &mut hashmap);
            });
        }

        let document = match self.document_map.get(uri.as_str()) {
            Some(rope) => rope,
            None => return Ok(None),
        };
        let inlay_hint_list = hashmap
            .into_iter()
            .map(|(k, v)| {
                (
                    k.start,
                    k.end,
                    match v {
                        jjmagit_language_server::jjmagit_lang::Value::Null => "null".to_string(),
                        jjmagit_language_server::jjmagit_lang::Value::Bool(_) => "bool".to_string(),
                        jjmagit_language_server::jjmagit_lang::Value::Num(_) => "number".to_string(),
                        jjmagit_language_server::jjmagit_lang::Value::Str(_) => "string".to_string(),
                    },
                )
            })
            .filter_map(|item| {
                // let start_position = offset_to_position(item.0, document)?;
                let end_position = offset_to_position(item.1, &document)?;
                let inlay_hint = InlayHint {
                    text_edits: None,
                    tooltip: None,
                    kind: Some(InlayHintKind::TYPE),
                    padding_left: None,
                    padding_right: None,
                    data: None,
                    position: end_position,
                    label: InlayHintLabel::LabelParts(vec![InlayHintLabelPart {
                        value: item.2,
                        tooltip: None,
                        location: Some(Location {
                            uri: params.text_document.uri.clone(),
                            range: Range {
                                start: Position::new(0, 4),
                                end: Position::new(0, 10),
                            },
                        }),
                        command: None,
                    }]),
                };
                Some(inlay_hint)
            })
            .collect::<Vec<_>>();

        Ok(Some(inlay_hint_list))*/
        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        /*let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let completions = || -> Option<Vec<CompletionItem>> {
            let rope = self.document_map.get(&uri.to_string())?;
            let ast = self.ast_map.get(&uri.to_string())?;
            let char = rope.try_line_to_char(position.line as usize).ok()?;
            let offset = char + position.character as usize;
            let completions = completion(&ast, offset);
            let mut ret = Vec::with_capacity(completions.len());
            for (_, item) in completions {
                match item {
                    jjmagit_language_server::completion::ImCompleteCompletionItem::Variable(var) => {
                        ret.push(CompletionItem {
                            label: var.clone(),
                            insert_text: Some(var.clone()),
                            kind: Some(CompletionItemKind::VARIABLE),
                            detail: Some(var),
                            ..Default::default()
                        });
                    }
                    jjmagit_language_server::completion::ImCompleteCompletionItem::Function(
                        name,
                        args,
                    ) => {
                        ret.push(CompletionItem {
                            label: name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(name.clone()),
                            insert_text: Some(format!(
                                "{}({})",
                                name,
                                args.iter()
                                    .enumerate()
                                    .map(|(index, item)| { format!("${{{}:{}}}", index + 1, item) })
                                    .collect::<Vec<_>>()
                                    .join(",")
                            )),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        });
                    }
                }
            }
            Some(ret)
        }();
        Ok(completions.map(CompletionResponse::Array))*/
        Ok(None)
    }

    async fn rename(&self, _params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        debug!("rename");
        /*let workspace_edit = || -> Option<WorkspaceEdit> {
            let uri = params.text_document_position.text_document.uri;
            let semantic = self.semantic_map.get(uri.as_str())?;
            let rope = self.document_map.get(uri.as_str())?;
            let position = params.text_document_position.position;
            let offset = position_to_offset(position, &rope)?;
            let reference_list = get_references(&semantic, offset, offset + 1, true)?;

            let new_name = params.new_name;
            (!reference_list.is_empty()).then_some(()).map(|_| {
                let edit_list = reference_list
                    .into_iter()
                    .filter_map(|range| {
                        let start_position = offset_to_position(range.start, &rope)?;
                        let end_position = offset_to_position(range.end, &rope)?;
                        Some(TextEdit::new(
                            Range::new(start_position, end_position),
                            new_name.clone(),
                        ))
                    })
                    .collect::<Vec<_>>();
                let mut map = HashMap::new();
                map.insert(uri, edit_list);
                WorkspaceEdit::new(map)
            })
        }();
        Ok(workspace_edit)*/
        Ok(None)
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        debug!("configuration changed!");
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        debug!("workspace folders changed!");

        let mut workspace_folders = self.workspace_folders.write().await;
        workspace_folders.retain(|folder| {
            !params
                .event
                .removed
                .iter()
                .any(|removed| &removed.uri == folder)
        });
        workspace_folders.extend(params.event.added.into_iter().map(|added| added.uri));
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        debug!("watched files have changed!");
    }

    async fn execute_command(&self, command: ExecuteCommandParams) -> Result<Option<Value>> {
        debug!("command executed: {}", command.command);

        let result = (async || match command.command.as_str() {
            commands::OPEN => {
                let [workspace, page, file_path] = command.arguments.as_slice() else {
                    return Err(anyhow!(
                        "wrong arguments to command {}: {:?}",
                        commands::OPEN,
                        command.arguments
                    ));
                };
                let workspace = workspace
                    .as_str()
                    .map(Path::new)
                    .ok_or_else(|| anyhow!("wrong parameter workspace: {:?}", workspace))?;
                let page = page
                    .as_str()
                    .and_then(pages::named)
                    .ok_or_else(|| anyhow!("wrong parameter page {:?}", page))?;
                let argument = value_as_option(file_path)
                    .map(|x| x.as_str().context("invalid parameter file_path"))
                    .transpose()?
                    .unwrap_or("");

                jjmagit_language_server::commands::open_page(workspace, page, &[argument])
                    .await
                    .map(|p| p.to_str().unwrap().to_owned())
            }
            other => Err(anyhow!("unknown command: {}", other)),
        })()
        .await;

        match result {
            Ok(res) => Ok(Some(res.into())),
            Err(e) => {
                log::error!("failed to run command: {e}");
                Ok(None)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
struct InlayHintParams {
    path: String,
}

#[allow(unused)]
enum CustomNotification {}
impl Notification for CustomNotification {
    type Params = InlayHintParams;
    const METHOD: &'static str = "custom/notification";
}
struct TextDocumentItem<'a> {
    uri: Url,
    text: &'a str,
    #[expect(dead_code)]
    version: Option<i32>,
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem<'_>) -> anyhow::Result<()> {
        let rope = ropey::Rope::from_str(params.text);
        self.document_map.insert(params.uri.to_string(), rope);

        let page_path = params
            .uri
            .to_file_path()
            .map_err(|()| anyhow!("Expected path, got url"))?;
        let (repo_path, page, arguments) = pages::path::parse_path(&page_path)?;
        let arguments: Vec<_> = arguments.iter().map(String::as_str).collect();

        let repo = Repo::detect(&repo_path)?.ok_or_else(|| anyhow!("no jj root found"))?;

        let mut out = PageWriter::default();
        page.render(&mut out, &repo, &arguments)?;
        let page = out.finish();
        if let Some(parent) = page_path.parent() {
            std::fs::create_dir_all(&parent)?;
        }
        std::fs::write(page_path, &page.text)?;

        let changed = page.text != params.text;
        debug!("on_change regenerated a different file: {}", changed);

        self.page_map.insert(params.uri.to_string(), page);

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: DashMap::new(),
        page_map: DashMap::new(),
        workspace_folders: Default::default(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}

fn position_to_offset(position: Position, rope: &Rope) -> Option<usize> {
    let line_char_offset = rope.try_line_to_char(position.line as usize).ok()?;
    let slice = rope.slice(0..line_char_offset + position.character as usize);
    Some(slice.len_bytes())
}

fn range_to_offset(range: Range, rope: &Rope) -> Option<std::ops::Range<usize>> {
    Some(position_to_offset(range.start, rope)?..position_to_offset(range.end, rope)?)
}

fn intersects(range1: &std::ops::Range<usize>, range2: &std::ops::Range<usize>) -> bool {
    range1.start <= range2.end && range2.start <= range1.end
}

fn value_as_option(val: &Value) -> Option<&Value> {
    match val {
        Value::Null => None,
        other => Some(other),
    }
}

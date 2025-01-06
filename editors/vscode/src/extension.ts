import {
  ExtensionContext,
  window,
} from "vscode";
import * as vscode from "vscode";

import {
  Executable,
  ExecuteCommandParams,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export async function activate(context: ExtensionContext) {
  const traceOutputChannel = window.createOutputChannel("jjmagit Language Server trace");
  const command = process.env.SERVER_PATH || "jjmagit-language-server";

  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
        RUST_LOG: "debug",
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  let clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "jjmagit" }],
    traceOutputChannel,
  };

  client = new LanguageClient("jjmagit-language-server", "jjmagit language server", serverOptions, clientOptions);
  client.start();

  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(onDidOpenTextDocument));
  context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(e => onDidChangeTextDocument(e.document)));
  context.subscriptions.push(vscode.commands.registerCommand("jjmagit.open.split", commandSplit));
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return;

  return client.stop();
}



async function onDidChangeTextDocument(document: vscode.TextDocument) {
  // await foldAll();
}

async function onDidOpenTextDocument(document: vscode.TextDocument) {
  if (document.languageId === 'jjmagit' || document.fileName.endsWith(".jjmagit") || document.fileName.endsWith(".jjmagit.git")) {
    // await vscode.commands.executeCommand("workbench.action.files.setActiveEditorReadonlyInSession");
  }
}

async function commandSplit() {
  let workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder found");
    return;
  }

  let args = {
    command: "open.split",
    arguments: [workspaceFolder]
  } satisfies ExecuteCommandParams;
  let response = await client.sendRequest("workspace/executeCommand", args) as string;

  let document = await vscode.workspace.openTextDocument(vscode.Uri.file(response));
  let editor = await vscode.window.showTextDocument(document);

  await foldAll();
}

async function foldAll() {
  await vscode.commands.executeCommand('editor.foldAll');
}
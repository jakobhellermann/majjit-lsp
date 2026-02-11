import { ExtensionContext, window } from "vscode";
import * as vscode from "vscode";

import {
  Executable,
  ExecuteCommandParams,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

type PageName = "status" | "annotate";
const allPages: PageName[] = ["status", "annotate"];

let client: LanguageClient;

let outputChannel: vscode.OutputChannel;
export async function activate(context: ExtensionContext) {
  outputChannel = window.createOutputChannel("jjmagit language server");
  const traceOutputChannel = window.createOutputChannel("jjmagit language server trace");
  const command = process.env.SERVER_PATH || "jjmagit-language-server";

  outputChannel.show();

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
    outputChannel,
    traceOutputChannel,
  };

  client = new LanguageClient(
    "jjmagit-language-server",
    "jjmagit language server",
    serverOptions,
    clientOptions,
  );
  client.start();

  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(onDidOpenTextDocument));
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((e) => onDidChangeTextDocument(e.document)),
  );

  let registerPage = (page: PageName, f: () => void) =>
    context.subscriptions.push(vscode.commands.registerCommand(`jjmagit.open.${page}`, f));

  registerPage("status", () => openPage("status"));
  registerPage("annotate", () => openPage("annotate", true));
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return;

  return client.stop();
}

async function onDidChangeTextDocument(document: vscode.TextDocument) {
  // await foldAll();
}

async function onDidOpenTextDocument(document: vscode.TextDocument) {
  if (
    document.languageId === "jjmagit" ||
    document.fileName.endsWith(".jjmagit") ||
    document.fileName.endsWith(".jjmagit.git")
  ) {
    // await vscode.commands.executeCommand("workbench.action.files.setActiveEditorReadonlyInSession");
  }
}

async function openPage(page: PageName, includePath: boolean = false) {
  let workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
  if (!workspaceFolder) {
    vscode.window.showErrorMessage("No workspace folder found");
    return;
  }

  let argument = null;
  if (includePath) {
    const currentlyOpen = vscode.window.activeTextEditor?.document?.uri;
    if (!currentlyOpen || currentlyOpen.scheme != "file") {
      return vscode.window.showErrorMessage("No editor open");
    }
    const filePath = currentlyOpen.fsPath;

    if (!filePath.startsWith(workspaceFolder)) {
      return vscode.window.showErrorMessage(
        `File '${filePath}' doesn't belong to workspace '${workspaceFolder}'`,
      );
    }
    argument = filePath.substring(workspaceFolder.length + 1);
  }

  let args = {
    command: "open",
    arguments: [workspaceFolder, page, argument],
  } satisfies ExecuteCommandParams;
  let response = await client.sendRequest("workspace/executeCommand", args);

  if (typeof response !== "string") {
    vscode.window.showErrorMessage("Could not execute command, check jjmagit LSP logs");
    outputChannel.show();
    return;
  }

  let document = await vscode.workspace.openTextDocument(vscode.Uri.file(response));
  let editor = await vscode.window.showTextDocument(document);

  await foldAll();
}

async function foldAll() {
  await vscode.commands.executeCommand("editor.foldAll");
}

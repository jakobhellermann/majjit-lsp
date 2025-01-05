import {
  ExtensionContext,
  window,
} from "vscode";

import {
  Executable,
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
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return;

  return client.stop();
}
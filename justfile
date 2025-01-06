build-lsp:
  file=$(cargo build --message-format=json | jq '. | select(.reason == "compiler-artifact") | select(.target.kind == ["bin"]) | .executable' -r); \
    mkdir -p ./editors/vscode/out; rm -f ./editors/code/out/jjmagit-language-server; cp "$file" ./editors/vscode/out/

build-extension-vscode:
  cd editors/vscode && npm run package

install-extension-vscode: build-extension-vscode
  code --install-extension /home/jakob/dev/jj/majjit-lsp/editors/vscode/jjmagit-language-server-0.0.1.vsix

lint:
  cargo clippy



semantic-tokens:
  jj config --color never list --include-defaults colors -T 'name.remove_prefix("colors.") ++ "\n"' \
    | tr -d '"' | awk '{print "\"" $NF "\"," }' | sort | uniq

clean:
  rm editors/vscode/out -fr
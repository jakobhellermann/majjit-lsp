# jjmagit-lsp

WIP exploration of a editor agnostic magit-style interface for [jujutsu](https://github.com/jj-vcs/jj), implemented as a language server using

- folding ranges
- goto definition
- code actions

Inspired by https://matklad.github.io/2024/12/13/majjit-lsp.html.

Currently not more than a prototype. The basic design of mapping jj templates into on-the-fly generated files with metadata for LSP actions is working.
Next steps:

- actually implementing code actions
- figuring out if zed can bind keyboard shortcuts to extension commands

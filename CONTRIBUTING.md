# Contributing / Como contribuir

## English

Thanks for helping improve VectorCraft CLI for OpenDeck. Bug reports, documentation fixes, test coverage, Property Inspector accessibility improvements, and focused code changes are welcome.

1. Search existing issues first; open one for a bug or a substantial proposal.
2. Fork the repository and work on a descriptive branch.
3. Keep changes Linux x86_64 compatible unless the issue explicitly concerns the Windows/macOS roadmap.
4. Run `cargo fmt --all -- --check`, `cargo test --all-targets`, and `bash scripts/package.sh` before opening a PR.
5. Explain the user-visible behavior, configuration examples, and how you tested it in the pull request.

Please do not add shell evaluation to the plugin. Arguments must remain a JSON string array passed directly to the executable. Do not commit `target/`, `dist/`, downloaded VectorCraft binaries, private project files, tokens, or control-port credentials.

The project welcomes public forks and pull requests. `LuizGustavoWT` is the only direct maintainer with merge/write and issue-management permissions. GitHub's permissions do not split “close issues” into a standalone role; contributors without write/triage access cannot close someone else's issue.

## Português do Brasil

Obrigado por ajudar a melhorar o VectorCraft CLI para OpenDeck. Relatos de bugs, correções de documentação, testes, melhorias de acessibilidade no Property Inspector e mudanças objetivas no código são bem-vindos.

1. Pesquise as issues existentes; abra uma para bugs ou propostas relevantes.
2. Faça um fork e trabalhe em uma branch com nome descritivo.
3. Mantenha compatibilidade com Linux x86_64, exceto quando a issue tratar explicitamente do roadmap Windows/macOS.
4. Antes do PR, execute `cargo fmt --all -- --check`, `cargo test --all-targets` e `bash scripts/package.sh`.
5. Explique no PR o comportamento percebido pelo usuário, exemplos de configuração e como você testou.

Não adicione avaliação por shell ao plugin. Os argumentos devem permanecer um vetor JSON de strings passado diretamente ao executável. Não envie `target/`, `dist/`, binários baixados do VectorCraft, arquivos privados de projeto, tokens ou credenciais de porta de controle.

Forks e pull requests públicos são bem-vindos. `LuizGustavoWT` é o único mantenedor com permissões diretas de merge/escrita e gerenciamento de issues. As permissões do GitHub não separam “fechar issues” em uma função isolada; contribuidores sem escrita/triagem não conseguem fechar issues de terceiros.

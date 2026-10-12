# VectorCraft CLI for OpenDeck

[Português do Brasil](#português-do-brasil) · [Contributing](CONTRIBUTING.md) · [VectorCraft](https://github.com/storytold/vectorcraft) · [OpenDeck](https://github.com/nekename/OpenDeck)

An independent Linux-only [OpenDeck](https://github.com/nekename/OpenDeck) plugin for running every available [VectorCraft CLI](https://github.com/storytold/vectorcraft) command from a Stream Deck key. It does not bundle VectorCraft: point it at the `vectorcraft-cli` installed on your machine.

Each action stores an executable path and a JSON array of arguments. The plugin launches the executable directly, without `sh -c` or another shell. That makes JSON parameters, spaces in paths, and untrusted filenames safe to pass as arguments.

## Features

- One configurable **Run VectorCraft CLI** action for keypad keys and encoders.
- Supports every VectorCraft CLI subcommand, option, and future addition: the action forwards the exact JSON argument array you configure.
- Loads the installed CLI's command or tool catalog in the Property Inspector and creates an app-specific, editable action template.
- Configurable executable path, working directory, timeout (1–300 seconds), label, and optional output summary on the key.
- Test a configuration in the Property Inspector before assigning it to a key.
- Linux x86_64 release ZIP created on every CI run; tagged versions are published as GitHub Releases.

## Requirements

- Linux x86_64
- OpenDeck with OpenAction-compatible plugin support
- VectorCraft and `vectorcraft-cli` installed and available in `PATH`, or its absolute path entered in the Property Inspector

## Install

1. Download `getcraft-vectorcraft-opendeck-linux-x86_64.zip` from the [latest release](../../releases/latest).
2. In OpenDeck, open **Plugins** → **Install from file** and select the ZIP. Do not extract or rename the `.sdPlugin` directory.
3. Drag **VectorCraft CLI → Run VectorCraft CLI** to a key.
4. Enter the CLI path if `vectorcraft-cli` is not in `PATH`, then enter a JSON argument array.
5. Use **Test command**. A green check means the CLI exited successfully; an alert means that its error summary is available in the Property Inspector and OpenDeck logs.

## Configuration and examples

The **Arguments** field must contain a JSON array of strings. It is not a shell command.

| Goal | Arguments |
| --- | --- |
| List the installed command catalog | `["commands"]` |

Use the exact argument syntax shown by `vectorcraft-cli --help` for project, file, export, and live-app control operations. The generic array accepts every documented subcommand and option.

The Property Inspector's **Load command catalog** button asks the installed CLI for the catalog supported by this application. Selecting an ID fills an app-specific, editable argument template; use **Test command** after adding the parameters and options required by your installed CLI version. The generic JSON array remains the canonical path to every CLI capability.

## CI/CD and release

Pull requests and pushes to `main` run formatting checks, unit tests, and `scripts/package.sh`. The workflow uploads the Linux installer ZIP as an artifact. Pushing a semantic tag such as `v0.1.0` publishes the same ZIP as a GitHub Release.

To package locally:

```bash
rustup target add x86_64-unknown-linux-gnu
bash scripts/package.sh
```

The output is `dist/getcraft-vectorcraft-opendeck-linux-x86_64.zip`, containing exactly `wt.luizgustavo.getcraft.vectorcraft.sdPlugin/` at its root.

## Local test installation

On Linux, build and install the plugin into OpenDeck's normal local plugin directory with:

```bash
bash scripts/install-local.sh
```

The installer stages the package, replaces only this plugin, and keeps any previous installation in a timestamped backup directory. It never restarts or reloads OpenDeck. When convenient, use OpenDeck's plugin reload control or restart it manually. To test against another OpenDeck configuration directory, set `OPENDECK_PLUGINS_DIR` before running the command.

## Platform roadmap

| Platform | Status |
| --- | --- |
| Linux x86_64 | Supported now |
| Windows | Planned; not implemented or packaged yet |
| macOS | Planned; not implemented or packaged yet |

## Security and governance

The plugin never evaluates a shell command. It executes the chosen binary with the strings in the JSON array, and it limits a process to five minutes.

This is an open-contribution repository. Direct write/merge access is reserved for `LuizGustavoWT`; outside contributors use forks and pull requests. GitHub does not have a separate permission that only controls closing issues: users without repository write/triage access cannot close other users' issues, so retaining those permissions only for the maintainer satisfies the governance model. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT. VectorCraft and OpenDeck are independent upstream projects with their own licenses and release policies.

---

# Português do Brasil

Plugin independente, somente para Linux, do [OpenDeck](https://github.com/nekename/OpenDeck) para executar todos os comandos disponíveis da [CLI do VectorCraft](https://github.com/storytold/vectorcraft) a partir de uma tecla do Stream Deck. O VectorCraft não é incluído no plugin: informe o `vectorcraft-cli` instalado na sua máquina.

Cada ação guarda o executável e uma lista JSON de argumentos. O plugin chama o executável diretamente, sem `sh -c` nem outro shell. Assim, JSON, caminhos com espaços e nomes de arquivos não confiáveis são passados com segurança.

## Recursos

- Ação configurável **Run VectorCraft CLI** para teclas e encoders.
- Todos os subcomandos, opções e futuras adições da CLI: a ação repassa exatamente o vetor JSON configurado.
- Catálogo dinâmico de comandos ou ferramentas da CLI no Property Inspector, com modelo de ação editável e específico do aplicativo.
- Caminho do executável, diretório de trabalho, timeout (1–300 s), rótulo e resumo de saída configuráveis.
- Botão de teste no Property Inspector antes de associar a ação a uma tecla.
- ZIP Linux x86_64 gerado em toda execução da CI; tags são publicadas como Releases.

## Requisitos e instalação

Você precisa de Linux x86_64, OpenDeck compatível com OpenAction e VectorCraft com `vectorcraft-cli` no `PATH` (ou o caminho absoluto no Property Inspector).

1. Baixe `getcraft-vectorcraft-opendeck-linux-x86_64.zip` na [release mais recente](../../releases/latest).
2. No OpenDeck, abra **Plugins** → **Install from file** e escolha o ZIP. Não extraia nem renomeie a pasta `.sdPlugin`.
3. Arraste **VectorCraft CLI → Run VectorCraft CLI** para uma tecla.
4. Configure o caminho da CLI, se necessário, e o vetor JSON de argumentos.
5. Use **Test command**. O check verde indica sucesso; o alerta mostra que houve erro, cujo resumo aparece no inspetor e nos logs do OpenDeck.

## Configuração

O campo **Arguments** precisa ser um vetor JSON de strings; ele não é um comando de shell.

| Objetivo | Arguments |
| --- | --- |
| Listar o catálogo de comandos instalado | `["commands"]` |

Use a sintaxe exata de `vectorcraft-cli --help` para operações de projeto, arquivo, exportação e controle do aplicativo em execução. O vetor genérico aceita todos os subcomandos e opções documentados.

**Load command catalog** consulta a CLI instalada pelo catálogo compatível com este aplicativo. Ao escolher um ID, o inspetor preenche um modelo de argumentos específico e editável; use **Test command** depois de adicionar os parâmetros e opções exigidos pela versão instalada da CLI. O vetor JSON genérico continua sendo o caminho para toda a CLI.

## CI/CD, roadmap e contribuição

PRs e pushes em `main` executam formatação, testes e `scripts/package.sh`. O ZIP vira artefato; uma tag como `v0.1.0` cria uma GitHub Release.

Para gerar e instalar o plugin localmente no Linux, execute `bash scripts/install-local.sh`. O instalador prepara a nova versão, substitui somente este plugin e preserva qualquer versão anterior em um diretório de backup. Ele nunca reinicia nem recarrega o OpenDeck; quando for conveniente, recarregue o plugin pelo OpenDeck ou reinicie manualmente. Para testar outra configuração, defina `OPENDECK_PLUGINS_DIR` antes do comando.

| Plataforma | Situação |
| --- | --- |
| Linux x86_64 | Suportado agora |
| Windows | Planejado; ainda não implementado nem empacotado |
| macOS | Planejado; ainda não implementado nem empacotado |

Contribuições são bem-vindas via fork e pull request. Apenas `LuizGustavoWT` possui acesso direto para merge e para fechar issues de terceiros. O GitHub não oferece uma permissão separada somente para fechar issues; sem acesso de escrita/triagem, colaboradores públicos não conseguem fechar issues de outras pessoas. Veja [CONTRIBUTING.md](CONTRIBUTING.md).

## Licença

MIT. VectorCraft e OpenDeck são projetos upstream independentes, cada um com sua própria licença.

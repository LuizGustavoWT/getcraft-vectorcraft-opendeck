# VectorCraft Controls for OpenDeck

[Português do Brasil](#português-do-brasil) · [Contributing](CONTRIBUTING.md) · [VectorCraft](https://github.com/storytold/vectorcraft) · [OpenDeck](https://github.com/nekename/OpenDeck)

An independent Linux-only [OpenDeck](https://github.com/nekename/OpenDeck) plugin for running every available [VectorCraft CLI](https://github.com/storytold/vectorcraft) command from a Stream Deck key. It does not bundle VectorCraft: point it at the `vectorcraft-cli` installed on your machine.

Each action stores an executable path and a JSON array of arguments. The plugin launches the executable directly, without `sh -c` or another shell. That makes JSON parameters, spaces in paths, and untrusted filenames safe to pass as arguments.

## Features

- **Open VectorCraft** starts the desktop app with the selected loopback host and control port, ready for direct document controls.
- **VectorCraft Live Control** is a real control-surface action for the currently open document. Its ready-made controls include Undo/Redo, selection and grouping, Pen/Text/Rectangle tools, a zoom encoder and a nudge encoder. It sends a local request to the running app, so it keeps the active document, selection and undo history intact.
- Encoders can send any documented control request on each turn. Their JSON supports numeric `$TICKS`, `$STEP`, and `$AMOUNT` tokens, making an Ulanzi D200 or another rotary deck useful for zooming, nudging, rotation, opacity and other numeric commands.

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
3. Drag **VectorCraft Controls → Open VectorCraft** to a key, set the desktop executable and control host/port, then press it. Drag **VectorCraft Live Control** to keys or encoders using the same endpoint.
4. The **Run VectorCraft CLI** action remains available for safe file conversion and repeatable headless batch jobs.
5. For batch jobs, enter the CLI path if `vectorcraft-cli` is not in `PATH`, then enter a JSON argument array and use **Test command**.

## Live-control recipes

The action only reaches `127.0.0.1`, but VectorCraft's control port has no authentication: enable it only while you are using your control surface. The bundled presets are the fastest way to get started:

| Deck control | Result |
| --- | --- |
| Undo, Redo, Select all, Deselect | Direct editing commands on the open document |
| Duplicate, Group, Ungroup | Selection workflow without leaving the canvas |
| Pen, Text, Rectangle | Switches the live application tool |
| Zoom encoder | Turn to zoom around the canvas; press runs a harmless state check |
| Nudge encoder | Turn to move the current selection one point per detent; change **Amount per click** to suit the job |

For additional app commands, select **Custom command**. Use `engine.execute` with a VectorCraft command ID and JSON object, or a protocol method such as `ui.tool.select`. An encoder's `$AMOUNT` expands to `ticks × amount per click` as a JSON number.

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

On Linux, build the local installation package with:

```bash
bash scripts/install-local.sh
```

By default, the installer only builds the ZIP and does not change OpenDeck's active plugin directory. This prevents development plugins from disrupting an active setup. To deliberately test one plugin in the active OpenDeck installation, opt in explicitly:

```bash
OPENDECK_ALLOW_LIVE_INSTALL=1 bash scripts/install-local.sh
```

Install one development plugin at a time, then use OpenDeck's plugin reload control or restart it manually. The installer itself never restarts or reloads OpenDeck. To target a separate configuration, set `OPENDECK_PLUGINS_DIR` before running the command.

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

Plugin independente, somente para Linux, do [OpenDeck](https://github.com/nekename/OpenDeck) para transformar o Stream Deck em uma superfície de controle do [VectorCraft](https://github.com/storytold/vectorcraft). Há controles ao vivo para o documento aberto e, quando necessário, automações de lote via `vectorcraft-cli`; o VectorCraft não é incluído no plugin.

Cada ação guarda o executável e uma lista JSON de argumentos. O plugin chama o executável diretamente, sem `sh -c` nem outro shell. Assim, JSON, caminhos com espaços e nomes de arquivos não confiáveis são passados com segurança.

## Recursos

- **Open VectorCraft** abre o aplicativo com o host loopback e a porta de controle escolhidos, pronto para os controles diretos do documento.
- **VectorCraft Live Control** opera diretamente o documento aberto: Undo/Redo, seleção, agrupamento, ferramentas Pen/Texto/Retângulo, encoder de zoom e encoder de nudge. A ação usa a porta de controle local, preservando documento, seleção e histórico de undo.
- Encoders podem disparar qualquer requisição documentada a cada giro. Os tokens numéricos `$TICKS`, `$STEP` e `$AMOUNT` servem para zoom, nudge, rotação, opacidade e outros ajustes graduais no Ulanzi D200 ou em outro deck com rotary.
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
3. Para edição cotidiana, arraste **VectorCraft Controls → Open VectorCraft** para uma tecla, configure executável, host e porta, depois pressione-a. Arraste **VectorCraft Live Control** para teclas ou encoders usando o mesmo endpoint.
4. A ação **Run VectorCraft CLI** continua disponível para conversão de arquivos e automações de lote seguras.
5. Para trabalhos em lote, configure o caminho da CLI, se necessário, o vetor JSON de argumentos e use **Test command**.

## Receitas de controle ao vivo

A ação só acessa `127.0.0.1`, mas a porta de controle do VectorCraft não possui autenticação: deixe-a ativa somente durante o uso da superfície de controle. Os controles prontos incluídos são:

| Controle do deck | Resultado |
| --- | --- |
| Undo, Redo, Selecionar tudo, Desmarcar | Comandos diretos no documento aberto |
| Duplicar, Agrupar, Desagrupar | Fluxo de seleção sem sair da tela de trabalho |
| Pen, Texto, Retângulo | Troca a ferramenta do aplicativo em execução |
| Encoder Zoom | Gire para dar zoom no canvas; pressionar faz uma consulta de estado inofensiva |
| Encoder Nudge | Gire para mover a seleção atual um ponto por clique; ajuste **Amount per click** conforme o trabalho |

Para outros comandos do aplicativo, escolha **Custom command**. Use `engine.execute` com o ID do comando VectorCraft e um objeto JSON ou um método do protocolo como `ui.tool.select`. Em um encoder, `$AMOUNT` vira o número JSON `ticks × amount per click`.

## Configuração

O campo **Arguments** precisa ser um vetor JSON de strings; ele não é um comando de shell.

| Objetivo | Arguments |
| --- | --- |
| Listar o catálogo de comandos instalado | `["commands"]` |

Use a sintaxe exata de `vectorcraft-cli --help` para operações de projeto, arquivo, exportação e controle do aplicativo em execução. O vetor genérico aceita todos os subcomandos e opções documentados.

**Load command catalog** consulta a CLI instalada pelo catálogo compatível com este aplicativo. Ao escolher um ID, o inspetor preenche um modelo de argumentos específico e editável; use **Test command** depois de adicionar os parâmetros e opções exigidos pela versão instalada da CLI. O vetor JSON genérico continua sendo o caminho para toda a CLI.

## CI/CD, roadmap e contribuição

PRs e pushes em `main` executam formatação, testes e `scripts/package.sh`. O ZIP vira artefato; uma tag como `v0.1.0` cria uma GitHub Release.

Para gerar o pacote local no Linux, execute `bash scripts/install-local.sh`. Por padrão, o instalador gera o ZIP e não altera o diretório de plugins ativo do OpenDeck, protegendo a configuração em uso. Para testar deliberadamente um único plugin na instalação ativa, confirme explicitamente:

```bash
OPENDECK_ALLOW_LIVE_INSTALL=1 bash scripts/install-local.sh
```

Instale um plugin de desenvolvimento por vez e então recarregue-o pelo OpenDeck ou reinicie manualmente. O instalador jamais reinicia ou recarrega o OpenDeck. Para usar outra configuração, defina `OPENDECK_PLUGINS_DIR` antes do comando.

| Plataforma | Situação |
| --- | --- |
| Linux x86_64 | Suportado agora |
| Windows | Planejado; ainda não implementado nem empacotado |
| macOS | Planejado; ainda não implementado nem empacotado |

Contribuições são bem-vindas via fork e pull request. Apenas `LuizGustavoWT` possui acesso direto para merge e para fechar issues de terceiros. O GitHub não oferece uma permissão separada somente para fechar issues; sem acesso de escrita/triagem, colaboradores públicos não conseguem fechar issues de outras pessoas. Veja [CONTRIBUTING.md](CONTRIBUTING.md).

## Licença

MIT. VectorCraft e OpenDeck são projetos upstream independentes, cada um com sua própria licença.

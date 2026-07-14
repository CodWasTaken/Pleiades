# Live terminal workspace

Running `pleiades` opens a full-screen Ratatui workspace. It is a persistent application, not a prompt/read/print loop: terminal input, model streams, agent events, background work, and a 20 FPS render tick are selected concurrently. Long model requests, commands, and permission decisions therefore do not stop input or redraws.

## Persistent regions

- **Header** — identity, provider/model, workspace, access mode, and task state.
- **Conversation** — Markdown user and agent messages with bounded, scrollable history.
- **Activity** — structured inspecting, searching, reading, planning, editing, executing, testing, reviewing, approval, completion, failure, and cancellation states.
- **Composer** — multiline input, selection, undo/redo, paste, history, slash completion, and follow-up queuing.
- **Status** — mode, Git branch/dirty state, usage, elapsed time, active operation, and keyboard hints.

Completed operations collapse into one-line activity records. Failures and approval requests remain prominent. Select an activity and use `Ctrl+T` for its details or `Ctrl+O` for captured output.

## Keyboard reference

| Key | Action |
|---|---|
| `Enter` | Send the composer; while work runs, queue it as a follow-up |
| `Alt+Enter` | Insert a composer newline |
| `Ctrl+Up` / `Ctrl+Down` | Navigate submitted input history |
| `Tab` | Complete a slash command or cycle focus |
| `PageUp` / `PageDown`, mouse wheel | Scroll without stopping live output |
| `Ctrl+C` | Cancel current work |
| `Ctrl+P` | Search the command palette |
| `Ctrl+R` / `Ctrl+M` | Select provider / model |
| `Ctrl+F` | Find and reference a workspace file |
| `Ctrl+L` | Open a saved session |
| `Ctrl+D` | Review the current Git diff |
| `Ctrl+O` / `Ctrl+T` | Open activity output / details |
| `Ctrl+,` | Open configuration information |
| `F1` | Search help |
| `Ctrl+Q` | Save and exit |

Slash commands, searchable help, the command palette, and nested completion are
generated from one typed command registry. Typing `/` and pressing `Tab` lists
root commands; typing a family followed by a space, such as `/mode `, lists its
registered subcommands. Palette selections and typed slash commands both travel
through the runtime command channel and return structured UI events—neither path
executes provider or tool logic in the TUI.

The currently implemented workspace commands include `/help`, `/status`,
`/mode`, `/provider`, `/model`, `/files`, `/sessions`, `/load`, `/diff`,
`/output`, `/doctor`, `/config`, `/clear`, `/save`, and `/quit`. Provider,
model, and plugin management subcommands will expand during Release 2.1.

## Terminal compatibility

The default `seven-sisters` theme uses restrained navy, blue-violet, cyan, and starlight tokens. `andromeda`, `orion`, `event-horizon`, `solar-wind`, `high-contrast`, and `ascii` are built in. The ASCII theme avoids Unicode symbols and rounded borders. Ratatui and Crossterm handle resizing and color capability fallback; no terminal background settings are modified.

Pleiades enters the alternate screen with raw input, bracketed paste, and mouse capture. A terminal guard restores raw mode, cursor visibility, mouse capture, bracketed paste, and the original screen on normal exit, errors, and panics.

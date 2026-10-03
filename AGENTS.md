# AGENTS.md

> This document is a navigation guide for AI/automation agents working in this repository. It summarizes the repository's overall architecture, the responsibility of each crate in the Cargo workspace, the boundaries of each submodule under the `app/` main binary, and the engineering conventions that must be followed before making changes.
>
> It is a companion to `WARP.md`: `WARP.md` is the engineer's handbook (commands, style, process), while this document is the **code map**. Read `WARP.md` first, then use this document to locate the correct crate / module.

---

## 1. Repository Overview

Warp is a Rust-centric **agentic terminal / development environment**: built on a self-developed UI framework (WarpUI), it integrates terminal emulation, an AI Agent, cloud sync (Drive), code review, completion, Notebooks, settings, IPC, and more.

Top-level directories:

| Directory | Purpose |
|------|------|
| `app/` | Main binary crate (`warp`), assembling all subsystems, UI, database migrations, and the platform glue layer |
| `crates/` | 67 workspace members, library crates split by responsibility |
| `command-signatures-v2/` | Standalone subproject (`--exclude`d when running nextest) |
| `script/` | Cross-platform bootstrap, build, and presubmit scripts |
| `resources/` | Runtime resources such as fonts, icons, shell integration scripts, and shaders |
| `docker/` | Containerized build related |
| `specs/` | Product/technical spec documents |
| `.agents/skills`, `.claude/skills` | Skill descriptions for agent workflows (creating PRs, fixing errors, feature rollout, etc.) |
| `.warp/`, `.config/`, `.cargo/`, `.vscode/` | Various tool configurations |

Build system: Cargo workspace, `resolver = "2"`, with `default-members` deliberately narrowed to the subset that frequently needs compiling/testing (see `Cargo.toml`). `serve-wasm` and `integration` are not in `default-members` by default.

License split:
- `crates/warpui` and `crates/warpui_core` → MIT
- Everything else → AGPL-3.0-only

---

## 2. Top-Level Architecture Layers

There are roughly 4 layers, from the bottom up. When adding code or locating a bug, first determine which layer the change belongs to, and **do not create inverted cross-layer dependencies**.

```
app/  (main binary: assembly, entry points, platform glue, persistence migrations, UI view root)
  ↑
Product-domain crates: ai / computer_use / vim / onboarding /
              warp_completer / lsp / languages / code-review …
  ↑
Framework crates: warpui / warpui_core / warpui_extras / editor /
            ui_components / sum_tree / syntax_tree
  ↑
Infrastructure crates: warp_core / warp_util / http_client /
                websocket / ipc / jsonrpc / persistence / graphql /
                managed_secrets / virtual_fs / watcher / asset_cache …
```

Key architectural patterns (see `WARP.md` for details):

1. **Entity-Handle system**: `App` globally owns all view/model entities; views reference each other through `ViewHandle<T>` rather than owning them directly.
2. **Element / Action**: the UI is composed of a declarative Element tree plus an Action event system (Flutter-style).
3. **Cross-platform**: native implementations for macOS / Windows / Linux plus a WASM target; platform code is isolated with `#[cfg(...)]`.
4. **AI integration**: Agent Mode and context indexing, with code concentrated in `app/src/ai` (389 files) and `crates/ai`.
5. **Cloud sync**: `Drive` keeps objects synchronized across devices; see `app/src/drive` and `crates/warp_files`.
6. **Feature Flags**: runtime rollout takes precedence over `#[cfg]`; the enum is defined in `crates/warp_core/src/features.rs`.

---

## 3. `crates/` at a Glance

The table below lists all 67 crates grouped by theme. Each row gives only a **one-sentence responsibility**; for implementation details, open the corresponding `crates/<name>/src/lib.rs` (many crates have `//!` module docs at the top of `lib.rs`).

### 3.1 UI Framework / View Layer

| Crate | Responsibility |
|-------|------|
| `warpui_core` | WarpUI framework core (MIT): infrastructure such as `App` / `Entity` / `ViewHandle` / `AppContext` |
| `warpui` | WarpUI higher-level components, Element tree, layout, render pipeline (MIT) |
| `warpui_extras` | Optional add-ons for WarpUI; not all features are enabled by default |
| `ui_components` | High-level component library reused across views (buttons, inputs, lists, modals, etc.) |
| `editor` (`warp_editor`) | Text editor: buffers, selection, cursor, keymaps, undo stack |
| `sum_tree` | Persistent balanced B-tree; the core data structure for the editor / Notebook / large lists |
| `syntax_tree` | Tree-sitter wrapper and syntax highlighting support |
| `markdown_parser` | Markdown parsing (used for AI messages, document views, Notebooks, etc.) |
| `vim` | Vim mode key bindings and operation semantics |
| `voice_input` | Voice input support |

### 3.2 Terminal

| Crate | Responsibility |
|-------|------|
| `warp_terminal` | Terminal emulation core: PTY management, ANSI/VT parsing, grid, scrolling, shell integration hooks |
| `input_classifier` | Terminal input intent classification (pure command / natural language / AI prompt) |
| `natural_language_detection` | Natural language detection (works together with `input_classifier`) |

### 3.3 AI / Agent

| Crate | Responsibility |
|-------|------|
| `ai` | AI model clients, prompt orchestration, agent protocol, tool-calling framework |
| `computer_use` | Rust-side implementation of "Computer Use" tool capabilities (screenshots, clicks, typing, etc.) |
| `command-signatures-v2` | Command signatures v2 (command classification metadata used by the AI); a standalone project that is not part of the main workspace test set |
| `onboarding` | New-user onboarding flow data/state |

### 3.4 Networking / Protocols / IPC

| Crate | Responsibility |
|-------|------|
| `http_client` | Workspace-wide unified HTTP client wrapper |
| `http_server` | Embedded HTTP server (local RPC, login callbacks, etc.) |
| `websocket` | Shared WebSocket abstraction for native and WASM, adapted to `graphql_ws_client` |
| `ipc` | Generic typed IPC request/response protocol (inter-process) |
| `jsonrpc` | JSON-RPC implementation |
| `lsp` | Language Server Protocol client implementation |
| `remote_server` | Server-side logic in remote (sshd) mode |
| `serve-wasm` | Helper server that hosts WASM build artifacts (not compiled by default) |
| `firebase` | Firebase client utilities (Crash/analytics channels, etc.) |

### 3.5 Persistence / Files / Resources

| Crate | Responsibility |
|-------|------|
| `persistence` | Diesel + SQLite persistence layer foundation; **migrations live in `app/migrations/`, and the schema is in `app/src/persistence/schema.rs`** |
| `warp_files` | Syncable file objects such as Drive files, Workflows, and Notebooks |
| `virtual_fs` | Abstract filesystem (mock for tests and the real FS for production share the same interface) |
| `repo_metadata` | Repository metadata: file tree construction, `.gitignore` handling, filesystem watching |
| `watcher` | Filesystem watcher (a wrapper around `notify`) |
| `asset_cache` | On-disk/in-memory asset cache |
| `asset_macro` | Asset reference macros such as `bundled!` / `theme!` |
| `managed_secrets` / `managed_secrets_wasm` | Keychain / DPAPI / Linux Keyring abstraction + WASM proxy |

### 3.6 Configuration / Settings

| Crate | Responsibility |
|-------|------|
| `settings` | Settings storage and change distribution |
| `settings_value` | The `SettingsValue` trait: controls TOML serialization semantics |
| `settings_value_derive` | `#[derive(SettingsValue)]` procedural macro (enum variants to snake_case, etc.) |
| `warp_features` | High-level Feature flag API (consumer side) |
| `channel_versions` | Release channels (stable/preview/dogfood) and version comparison |

### 3.7 Commands / Completion / Languages

| Crate | Responsibility |
|-------|------|
| `command` | Safe wrapper for cross-platform process spawning, **with special handling for the Windows `no_window` flag**; always route new child process spawns through here |
| `warp_completer` | Completion engine (supports `--features v2`) |
| `languages` | Language/extension/Tree-sitter grammar registration |
| `warp_ripgrep` | Thin ripgrep wrapper used by `warp_cli` |
| `warp_cli` | CLI subcommand parsing inside the binary (`warp <subcmd>`) |
| `fuzzy_match` | Fuzzy matching plus glob-style wildcards, used for path search and the command palette |

### 3.8 Platform / System Services

| Crate | Responsibility |
|-------|------|
| `app-installation-detection` | Detects apps already installed on the system (for launcher integration) |
| `prevent_sleep` | Sleep inhibition (during long tasks / AI Agent runs) |
| `isolation_platform` | Compatibility layer for running in sandboxes such as Docker / GitHub Actions |
| `node_runtime` | Automatically installs/manages Node.js and npm (macOS/Linux/Windows × multiple architectures) |
| `warp_js` | Helper abstractions for manipulating JavaScript values/functions from the Rust side |

### 3.9 General Utilities / Communication

| Crate | Responsibility |
|-------|------|
| `warp_core` | The lowest-level "core" in the workspace: platform abstractions, the `FeatureFlag` enum in `features.rs`, and `DOGFOOD/PREVIEW/RELEASE_FLAGS` |
| `warp_util` | General utility functions reused across multiple crates |
| `warp_logging` | Unified entry point for logging configuration |
| `simple_logger` | Simple async file logging for stderr-only processes such as `remote_server` |
| `warp_web_event_bus` | Web-side event bus (for embedded web views) |
| `field_mask` | gRPC/Proto-style FieldMask utilities |
| `string-offset` | Offset primitive types (byte/char/utf16) |
| `handlebars` | Handlebars template engine wrapper |
| `integration` | Integration test framework, used only for tests |

> Naming gotchas: the package name of `crates/editor` is `warp_editor`; `crates/isolation_platform` is `warp_isolation_platform`; `crates/managed_secrets` is `warp_managed_secrets`; `crates/virtual_fs` is `virtual-fs` (hyphenated); `crates/string-offset` is `string-offset` (hyphenated).

---

## 4. `app/` Submodule Navigation

`app/src/` contains 60+ flat product-domain directories, each roughly corresponding to a product feature line. They are grouped by theme below, with the approximate number of `.rs` files in parentheses to help estimate module size:

### 4.1 Startup / Assembly / Global
- `bin/` (7) — multiple binary entry points (main program, auxiliary tools).
- `lib.rs` / `app_state.rs` / `app_state_tests.rs` — application state root.
- `app_menus.rs`, `app_services/`, `app_id_test.rs`
- `appearance.rs`, `gpu_state.rs`, `font_fallback.rs`, `global_resource_handles.rs`
- `dynamic_libraries.rs`, `alloc.rs`, `tracing.rs`, `profiling.rs`
- `crash_recovery.rs`, `crash_reporting/` (4)
- `features.rs` — consumption of `warp_core::FeatureFlag` inside `app/`; when adding a flag you usually need to wire it up in both places.
- `channel.rs`, `download_method.rs`, `autoupdate/` (8)

### 4.2 Terminal
- `terminal/` (427) — the main body: shell processes, PTY, grid, blocks, shell integration, command execution, I/O pipeline.
- `default_terminal/` (2) — default terminal launch logic.
- `shell_indicator.rs`, `prefix.rs` / `prefix_test.rs` (command prefix parsing), `vim_registers.rs`

### 4.3 AI / Agent
- `ai/` (389) — includes Agent UI, conversation models, agent management, tools/MCP, Cloud Agent, Plan/Diff views, artifacts, blocklist, execution profiles, etc. **This is the largest subtree in the repository**; before changing it, grep for the specific subtopic within that directory (`agent_*`, `conversation_*`, `cloud_agent_*`, `mcp`, `tool_*`).
- `ai_assistant/` (9) — legacy AI assistant entry points/adapters.
- `chip_configurator/`, `context_chips/` (22) — Agent context chip selection/construction.
- `coding_entrypoints/` (5), `coding_panel_enablement_state.rs`
- `prompt/` (2), `tips/` (3), `voice/` (2), `completer/` (3)

### 4.4 Editor / Code / Review
- `editor/` (38) — main editor integration.
- `code/` (52) — code views, diff, navigation.
- `code_review/` (36) — Code Review flow.
- `notebooks/` (30), `workflows/` (22)

### 4.5 Search
- `search/` (172) — multi-target search (files, commands, agent history, etc.).
- `search_bar.rs`

### 4.6 Server Communication / Drive / Sync
- `server/` (55) — HTTP/WS interaction with the warp backend (corresponds to local development mode `with_local_server`).
- `drive/` (45) — entry point for cloud object sync.
- `cloud_object/` (12) — cloud object abstraction layer (workflow, notebook, etc.).
- `remote_server/` (5) — client-side glue for connecting to remote-mode sshd.

### 4.7 Settings / User Config / Themes / Onboarding
- `settings/` (46), `settings_view/` (63)
- `user_config/` (6), `themes/` (11), `appearance.rs`
- `experiments/` (7), `tab_configs/` (15), `launch_configs/` (4)
- `tips/`, `banner/` (3), `quit_warning/` (1), `wasm_nux_dialog.rs`, `referral_theme_status.rs`

### 4.8 Auth / Billing / Usage
- `auth/` (22) — login, tokens, SSO.
- `billing/` (3), `pricing/` (1), `usage/` (1), `reward_view.rs`

### 4.9 Persistence
- `persistence/` (9) — Diesel migrations wiring, `schema.rs` (generated by Diesel), migration runner.
- Migration files live in the repository's top-level `migrations/` directory (managed by the Diesel CLI).

### 4.10 Platform / System Integration
- `platform/` (2), `system/` (3) / `system.rs`
- `login_item/` (3), `antivirus/` (3), `network.rs`
- `external_secrets/` (1), `env_vars/` (14)
- `keyboard.rs` / `keyboard_test.rs`, `safe_triangle.rs` / `safe_triangle_tests.rs` (menu hover safe triangle)

### 4.11 View Root / Panels / General UI
- `root_view.rs` / `root_view_tests.rs`
- `pane_group/` (35) — split-pane / split-block layout.
- `tab.rs`, `command_palette.rs`, `modal.rs`, `menu.rs` / `menu_test.rs`
- `palette.rs`, `notification.rs`, `resource_center/` (10)
- `view_components/` (20), `ui_components/` (14)
- `workspace/` (54), `workspaces/` (10), `voltron.rs` (multi-window / multi-workspace coordination)
- `session_management.rs`, `undo_close/` (3), `word_block_editor.rs`
- `suggestions/` (2), `input_suggestions.rs` / `input_suggestions_test.rs`
- `plugin/` (21) — plugin system integration.
- `uri/` (7) — `warp://` URL handling.
- `debug_dump.rs`, `debounce.rs`, `interval_timer.rs`, `throttle.rs`
- `linear.rs`, `resource_limits.rs`, `warp_managed_paths_watcher.rs`
- `preview_config_migration.rs` / `preview_config_migration_tests.rs`
- `window_settings.rs`, `projects.rs`

### 4.12 Test Infrastructure
- `integration_testing/` (79) — end-to-end integration test support.
- `test_util/` (6) — shared utilities for unit tests.

---

## 5. Engineering Discipline (Hard Constraints for Agents)

> These are compiled from `WARP.md` and project-specific rules; for the purposes of this document, the verification requirement for agents is `cargo check`.

### 5.1 Required Conventions
- **Comments/replies must be written in Simplified Chinese** (user rule).
- For searching/grepping within the git index, use the `fff` tool or `rg -n "<keyword>" <path>`; use `read_file` only for images/binaries.
- Before opening a PR / pushing a new commit, **only** this needs to pass: `cargo check`.
- Changes must be precise: **every changed line must trace back to the user's request**. Do not opportunistically "improve" unrelated code, comments, or formatting.
- Prefer simplicity: do not introduce abstractions, configuration, error handling, or extra features for a single use site.
- Explain options and surface uncertainty rather than silently making choices on the user's behalf.
- Worktree path: `.worktrees/<worktree_name>/`

### 5.2 Rust Style (excerpted from `WARP.md`)
- Do not add redundant type annotations to closure parameters.
- Use a single `use` block at the top; do not write long path-qualified expressions. Exception: inside `#[cfg]` branches.
- Name the context parameter `ctx` and put it last; if there is also a closure parameter, the closure goes last.
- **Delete** unused parameters outright rather than prefixing them with `_`, and update call sites accordingly.
- Use inline format arguments for macros such as `println!` / `format!` (`"{x}"` instead of `"{}", x`) to satisfy `uninlined_format_args`.
- **Do not use the `_` wildcard** in `match` statements (unless genuinely necessary); keep matches exhaustive.
- Do not delete or modify existing comments because of unrelated changes.

### 5.3 Terminal Model Lock (High Priority!)
- Calling `TerminalModel::lock()` deadlocks very easily (on macOS it manifests as a frozen UI / spinning beachball).
- Before adding a new `model.lock()`, you must confirm that no caller higher in the call stack already holds the lock; prefer passing an already-locked reference down the call stack instead of locking again.
- Minimize the lock scope; do not call functions that might lock again while holding the lock.

### 5.4 Feature Flags
- Adding one: add a variant to the `FeatureFlag` enum in `crates/warp_core/src/features.rs`; add it to `DOGFOOD_FLAGS` / `PREVIEW_FLAGS` / `RELEASE_FLAGS` as appropriate.
- Using one: **prefer** the runtime `FeatureFlag::Xxx.is_enabled()` over `#[cfg(...)]`; use `cfg` only when the code cannot compile without it (platform-specific / optional dependencies).
- Wrap an entire product feature rather than adding the check at every call site; once rollout is stable, **clean up the flag and dead branches**.
- UI entry points must use the same flag as the code path.

### 5.5 Database
- ORM: Diesel + SQLite.
- Adding/changing the schema must go through a migration: add a new directory under `migrations/` (`up.sql` / `down.sql`). Do not hand-edit `app/src/persistence/schema.rs` (it is generated by `diesel print-schema`).

### 5.6 Testing
- Use `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`.
- Put unit tests in `${filename}_tests.rs` or `mod_test.rs`, and add this at the end of the original file:

  ```rust
  #[cfg(test)]
  #[path = "filename_tests.rs"]
  mod tests;
  ```

- For integration tests, use the framework in `crates/integration`; examples are in `app/src/integration_testing/`.

### 5.7 Cross-Process Commands
- Do not call `std::process::Command::new(...)` directly (especially on Windows, where it can pop up a window); route everything through `crates/command`.

### 5.8 Subagents / Multi-Agent
- Split large tasks into subtasks with **non-overlapping write domains** and dispatch them in parallel; information-gathering tasks can also be parallelized.
- Do simple tasks directly; do not over-decompose.

---

## 6. Common Entry Points Quick Reference

| What you want to do | Starting point |
|---------|--------|
| Change the terminal grid / shell integration | `crates/warp_terminal/src/`, in conjunction with `app/src/terminal/` |
| Change the Agent UI / conversation | Grep by subtopic (`agent_*` / `conversation_*`) inside `app/src/ai/` |
| Change command completion | `crates/warp_completer/` (note `--features v2`) |
| Change AI models / tool-calling protocol | `crates/ai/` |
| Add a new setting | `crates/settings_value*`, `crates/settings`; UI is in `app/src/settings_view/` |
| Add a Feature Flag | `crates/warp_core/src/features.rs` + the usage sites |
| Change cloud sync objects | `crates/warp_files` + `app/src/drive/` + `app/src/cloud_object/` |
| Change the persistence structure | Add a migration under `migrations/` + `crates/persistence` |
| Add a new binary tool | `app/src/bin/` |
| Platform-specific code | Use `#[cfg(target_os = "...")]`; UI platform glue is in `app/src/platform/` |
| Vim mode | `crates/vim` + `app/src/vim_registers.rs` |
| Notebook / Workflow | `app/src/notebooks/`, `app/src/workflows/`, `crates/warp_files` |
| Cross-platform process spawning | `crates/command` |
| File search / watching | `crates/repo_metadata`, `crates/watcher`, `crates/warp_ripgrep` |

---

## 7. Pre-Change Checklist

Before you start typing code, ask yourself:

1. Which layer / crate / `app/src/<submodule>` does this belong to? Will the change cross a layer boundary?
2. Do you need a new dependency? If an existing workspace dependency can be reused, prefer reusing `[workspace.dependencies]` in `Cargo.toml`.
3. Is this a product feature? Does it need to be wrapped in a Feature Flag?
4. Does it involve the terminal model? Does the current call stack already hold the `TerminalModel` lock?
5. Does it involve child processes? Is it going through `crates/command`?
6. Does it involve persistence? Is a migration needed?
7. Have you written the corresponding `${file}_tests.rs`?
8. Is `cargo check` green?
9. Can every changed line be mapped one-to-one to the user's request? Should any opportunistic "small refactor" be rolled back?

Go through all 9 items above before delivering.
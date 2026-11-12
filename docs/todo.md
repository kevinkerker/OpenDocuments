# TODO

Open work for this fork (`kevinkerker/OpenDocuments`). Status as of 2026-11-12.
Order is priority: make OpenDocuments usable on a laptop first, then switch the project to English only.

## 0. Security before first use

- [x] **Any website can read and delete documents.** Fixed: CORS removed, `Host`/`Origin` guard, `api_key` required off loopback (`crates/opendoc-mcp/src/access.rs`). `crates/opendoc-mcp/src/lib.rs:835` sets `CorsLayer::permissive()` and no route checks `server.api_key`, so a page open in the browser can call `http://127.0.0.1:3000/api/v1/*` and read the responses. The WebUI is same-origin, so CORS is not needed. Remove it, reject foreign `Host`/`Origin` headers (DNS rebinding), and require `api_key` when `--host` is not loopback.
- [ ] **Provider API keys are stored in plain text** in `opendocuments.db`; README claims "encrypted … `600` permission", but no encryption exists. Encrypt them (Windows DPAPI / OS keyring) or correct the README before adding cloud keys.

## 1. Make it usable

- [ ] **`opendoc stop` does nothing.** `Commands::Stop` in `crates/opendoc-cli/src/main.rs` only prints a message; the server keeps running. Decide how `stop` finds the server (PID file next to the database vs. local shutdown endpoint), then implement and test.
- [ ] **Engine console window / orphan process on Windows.** `crates/opendoc-storage/src/sidecar_client.rs` spawns `opendoc-engine-lancedb` without `CREATE_NO_WINDOW` and without kill-on-parent-close, as required by `openspec/specs/lancedb-engine-sidecar`. A hidden `opendoc` (Task Scheduler) may open a console window; a hard crash may leave the engine running.
- [x] **Remove the legacy npm package** (removed 2026-11-12) `opendocuments` 0.3.0 (TypeScript original, `npm uninstall -g opendocuments`). It uses the same `~/.opendocuments/opendocuments.db` and port 3000 and would corrupt the Rust database if started.
- [ ] **`cargo install --git` builds without the WebUI.** `apps/webui/dist` is not in the repository, so the cargo install paths in README and docs-site likely produce a binary without a UI. Verify; then either document `make install` only or embed a fallback.
- [ ] **`make run` calls `serve`**, the command is `start`.
- [ ] **Scanned PDFs are not searchable.** The PDF parser uses `lopdf` text extraction only, with no OCR. Decide whether OCR is needed for the target documents.
- [ ] **WebUI uploads keep no original.** The file is parsed from a temp copy and deleted; citations show only the file name. `opendoc document index <folder>` keeps absolute paths. Decide the recommended ingest path (folder index vs. upload) and whether citations should open the source.
- [ ] **Autostart on Windows** (Task Scheduler per `docs-site/guide/deployment.md`) depends on the engine console-window fix above.
- [ ] **Backup** of `~/.opendocuments` and `~/.config/opendocuments` is not covered.

## 2. Release and distribution

- [ ] Push `main` and tag a release (e.g. `v1.0.1`). This is the first real run of the Windows job in `.github/workflows/release.yml` (not validated locally).
- [ ] Verify `install.ps1` and `install.sh` against the published release.
- [ ] Enable GitHub Pages for the fork; docs links point to `kevinkerker.github.io/OpenDocuments`.
- [ ] Independent review of `windows-install` against `openspec/changes/windows-install` (acceptance criteria), report in `docs/pruefung/`.
- [ ] When archiving `windows-install`: raise the `embedding-fastembed` budget in `openspec/specs/binary-size-architecture` to 35 MiB (release builds only).

## 3. English only

Goal: every user-facing string, doc, comment and spec in English; remove all other languages (currently Traditional Chinese, plus German in `openspec/changes/windows-install`). Inventory: 83 tracked files, 2,365 lines with Chinese text.

- [ ] **WebUI:** drop the zh-TW locale from `apps/webui/src/lib/i18n.ts` (387 lines) and the locale switch; translate remaining strings in `lib/display.ts`, `stores/`, `components/` (9 files).
- [ ] **CLI output:** `crates/opendoc-cli/src/main.rs` (165 lines), `Cargo.toml` description.
- [ ] **Server and libraries:** `opendoc-mcp` (18 files, 471 lines), `opendoc-storage` (5 files, 123 lines), `opendoc-connector-clouddrive`, `opendoc-llm`, `opendoc-types`, parsers, engine. Covers error messages, log lines and comments; check tests that assert on message text.
- [ ] **Docs:** delete `README.zh-TW.md`, `docs/zh-TW/`, `docs/ref/zh-TW/`; translate `docs/OpenDocuments-Requirements.md` (172 lines) and fix stray lines in `docs/en/`, `README.md`. Move the open entries of `docs/zh-TW/tasks.md` into this file.
- [ ] **Specs:** translate `openspec/specs` (5 files) and `openspec/changes` (5 files, including the German `windows-install` change).
- [ ] **Scripts:** `install.sh`, `Makefile`, `scripts/audit-docs.sh`.
- [ ] Add a check (script or CI step) that fails on non-English text in tracked files.

## 4. Models (see `docs/analyse/llm-models.md`)

- [ ] **Model settings** (`openspec/changes/model-settings`): global chat/embedding settings in the WebUI, Ollama model list and pull, query instruction, background re-embed on switch, Ollama-first defaults (`qwen3-embedding:0.6b`).
- [ ] Add an optional cross-encoder rerank stage (`bge-reranker-v2-m3` via fastembed); drop the unused `bge-reranker-base` default.
- [ ] Update stale model names (README, WebUI placeholder, Ollama guide, task-execution spec); fix or relabel the "Anthropic" provider type (client speaks the OpenAI protocol only).
- [ ] Watch harrier-oss-v1-0.6b (MIT, 69.0 MMTEB v2) for Ollama/fastembed availability.

## 5. Housekeeping

- [ ] Three `opendoc-mcp` tests (`test_admin_stats_200`, `test_list_workspaces_returns_real_data`, `test_stats_endpoint`) depend on `default_workspace = "homelab"` in the real `~/.config/opendocuments/config.toml`, and `test_config_manager_load_or_init` writes to the real config. Make them use a temporary home.
- [ ] Run `cargo clippy` on the workspace and fix new warnings.

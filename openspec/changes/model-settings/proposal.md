# Proposal

## Why

Switching models is hard today (2026-11-12):

- **Chat providers** are free-text rows in `llm_providers`: base URL plus model name typed by hand. The WebUI cannot list or download models.
- **The embedding model** lives in `config.toml` (`embedding_backend`, `embedding_provider_name`, `embedding_dim`, `embedding_table_name`). It is read once at startup, and changing it silently breaks search, because the LanceDB table keeps vectors from the old model.
- **Queries and documents are embedded the same way.** Current embedders (Qwen3-Embedding, harrier) require a task instruction on queries.
- The default embedder BGE-M3 is no longer state of the art (see `docs/analyse/llm-models.md`), and Ollama on the same machine is used only as an anonymous OpenAI-compatible URL.

No documents have been indexed on the target install yet, so defaults and storage layout can change freely.

## What Changes

- **Global model settings:** one chat model and one embedding model per instance, chosen in the WebUI and stored in the database. `config.toml` only seeds the first start.
- **Ollama integration:**
  - The server detects Ollama on `127.0.0.1:11434`.
  - It lists installed models split into chat and embedding (`/api/tags`, `/api/show`).
  - A short recommended list can be downloaded from the WebUI with progress (`/api/pull`).
- **Query/document split:** an optional query instruction per embedding model. Recommended models come with presets for the instruction and the dimension.
- **Background re-embedding:** one LanceDB table per embedding model. Switching re-embeds all chunks from the SQLite FTS5 store into the new table; search keeps using the old table until the new one is complete, then switches.
- **Defaults for new installs:** Ollama first.
  - Embedding: `qwen3-embedding:0.6b` (offered for download).
  - Chat: chosen from the installed Ollama models.
  - Without Ollama, the built-in BGE-M3 (fastembed) stays the fallback.

## Capabilities

### New Capabilities

- `model-settings`: global chat and embedding model selection, Ollama discovery and pull, query instructions, background re-embedding.

### Modified Capabilities

- `windows-distribution`: the first-start fallback writes the new global setting instead of `embedding_backend` in `config.toml`.

## Impact

- `crates/opendoc-storage`: settings table, per-model table naming, re-embed job, FTS5 chunk export.
- `crates/opendoc-llm`: embedding with query instruction, Ollama client (`/api/tags`, `/api/show`, `/api/pull`).
- `crates/opendoc-cli`: build the search backend from the settings; swappable backend.
- `crates/opendoc-mcp`: swappable `McpState.search`, settings and Ollama endpoints, re-embed status.
- `apps/webui`: settings section "Models".
- Docs: configuration, Ollama guide, README.

# Design

Status 2026-11-12. Decisions made with the user in a Q&A round. Each lists the options considered.

## Decisions

### D1 Abstraction: global admin setting

- **Chosen:** one chat model and one embedding model per instance, editable in the WebUI and stored in the database. This follows Open WebUI (Admin Settings > Documents) and AnythingLLM (embedder preference per instance). `config.toml` seeds the first start only.
- Rejected:
  - Model roles with a reranker slot and a model registry.
  - Per-workspace models with one LanceDB table per workspace.
  - Chat-only dropdown with the embedding model left in `config.toml`.
- **Consequence:** there is no general model registry. Dimension and query instruction are fields of the embedding setting, prefilled from the recommended list (D2) or from Ollama's `/api/show`.
- Chat credentials stay in `llm_providers`; the global setting points at one provider row and a model name.

### D2 Ollama: list installed models plus pull button

- **Chosen:**
  - The server probes `http://127.0.0.1:11434`, lists models via `/api/tags`, and classifies them via `/api/show` (`capabilities` contains `embedding` or `completion`; `model_info.*.embedding_length` gives the dimension).
  - A short recommended list can be pulled through `/api/pull`, with streamed progress shown in the WebUI. Each entry carries its presets: model, dimension, query instruction.
  - Initial list: `qwen3-embedding:0.6b` (embedding, 1024, instruction), `bge-m3` (embedding, 1024, no instruction), `gemma4:12b` and `qwen3:8b` (chat). The list is kept in code, next to the presets.
- Rejected:
  - Installed models only, with no pull.
  - OpenDocuments managing Ollama itself (install, start, auto-pull).
  - URL field only.
- Ollama calls go through the OpenDocuments server, not from the browser, so CORS and remote access do not matter.

### D3 Switching the embedding model: background re-embed

- **Chosen:**
  - **Table per model.** The LanceDB table name derives from the embedding identity: `od_chunks_<slug(provider/model)>_<dim>`.
  - **Switching** starts a job. A second engine process opens the new table; all chunks are read from `chunks_fts` (content, headers, path, index) and embedded in batches (documents without instruction). `collection_id` comes from `documents`.
  - **Search during the job** uses the old backend. When the job completes, `McpState` swaps the backend atomically and the old table is dropped.
  - **Progress** (chunks done / total, errors) is visible in the WebUI. A failed job leaves the old model active.
  - An empty index switches immediately.
- Rejected:
  - Confirm, wipe, re-embed (AnythingLLM); search finds nothing until it is done.
  - Parallel indexes for side-by-side comparison.
  - Switching only while the index is empty.
- New uploads during a job are written to both backends, so nothing is missing after the swap.

### D4 Defaults for new installs: Ollama first

- **Chosen:**
  - **Ollama reachable:** offer `qwen3-embedding:0.6b` (Multilingual MTEB v2 64.3, 1024 dim, Apache 2.0) for embedding, and let the user choose chat from the installed models.
  - **No Ollama:** the built-in BGE-M3 (fastembed) stays the fallback from `windows-install`, and there is no chat until a provider is configured.
- Rejected:
  - Built-in embedder always.
  - Built-in Qwen3 through fastembed's candle backend (unmeasured size and speed).
  - BGE-M3 everywhere.
- The first start never blocks on Ollama. If no embedding model is configured yet, the fallback applies, and the WebUI offers to switch, which triggers D3.

## Query instruction

- Embedding providers get `embed_documents` and `embed_query`. `embed_query` prepends the configured instruction.
- Qwen3-Embedding format: `Instruct: Given a question, retrieve passages that answer it\nQuery: {query}`.
- No instruction means identical behavior to today (BGE-M3).

## Risks / Trade-offs

- **Two engine processes** during a job double the memory use of LanceDB for that time.
- **Ollama classification** depends on `/api/show` returning `capabilities` (Ollama ≥ 0.6). Older versions show all models as unclassified.
- **Chat setting moves from per-workspace to global.** Existing per-workspace active flags are ignored; on migration, the first active provider becomes the global chat setting.
- **Reranking** stays out of scope (separate change).

## Migration

- Existing `config.toml` keys seed the settings row on first start after upgrade. The current table name `od_chunks` is adopted as the table of the seeded embedding model, so no re-embed is needed.

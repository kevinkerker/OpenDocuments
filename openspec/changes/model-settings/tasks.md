# Tasks

## 1. Backend foundations

- [ ] 1.1 Settings table (`model_settings`, single row) with chat and embedding settings; seed from `config.toml`; adopt `od_chunks` for the seeded embedder
- [ ] 1.2 `EmbeddingProvider`: `embed_documents` / `embed_query` with optional instruction (BYOK, fastembed)
- [ ] 1.3 Swappable search backend in `McpState` (atomic swap); build backend from settings instead of `config.toml`
- [ ] 1.4 Per-model LanceDB table naming

## 2. Ollama

- [ ] 2.1 Ollama client: probe, `/api/tags`, `/api/show` (capabilities, embedding_length), `/api/pull` streaming
- [ ] 2.2 Recommended model list with presets (model, kind, dimension, instruction)
- [ ] 2.3 API endpoints: list, pull with progress (SSE), settings get/put

## 3. Re-embed job

- [ ] 3.1 Export chunks from `chunks_fts` plus `collection_id` from `documents`
- [ ] 3.2 Background job: second engine process, batched embedding, progress, failure keeps old model
- [ ] 3.3 Dual-write uploads during a job; swap and drop old table on completion

## 4. WebUI

- [ ] 4.1 Settings section "Models": chat and embedding pickers (Ollama installed / recommended / OpenAI-compatible / built-in)
- [ ] 4.2 Pull progress, re-embed progress, Ollama-unreachable state

## 5. Verification and docs

- [ ] 5.1 Tests per requirement; end-to-end on this laptop: pull `qwen3-embedding:0.6b`, index, switch from BGE-M3, search during and after the job
- [ ] 5.2 Update configuration docs, Ollama guide, README; adjust `windows-install` first-start fallback

# Models used in OpenDocuments vs. state of the art

Status 2026-11-12. Checked at commit `ca3169c`: `crates/opendoc-storage` (config defaults, fastembed), `crates/opendoc-llm`, `crates/opendoc-cli`, `crates/opendoc-ai`, `apps/webui/src/components/settings`, README, docs-site, `openspec/specs/task-execution-ai-engines`.
Not checked: retrieval quality on a real corpus (no benchmark run), the live MTEB leaderboard (scores below come from model cards and papers; third-party blogs only where marked).

## What the project actually uses

| Role | Model | Where | Actually used at runtime? |
|---|---|---|---|
| Embedding (default) | **BGE-M3** (BAAI, Jan 2024), 1024 dim | `embedding_provider_name = "bge-m3"`, `embedding_dim = 1024` (`opendoc-storage/src/lib.rs`); fastembed `BGEM3` int8 ONNX (`gpahal/bge-m3-onnx-int8`) for the first-start fallback | Yes. Every vector in LanceDB comes from it (BYOK name or fastembed). |
| Reranker | **bge-reranker-base** (BAAI, 2023, English/Chinese) | `local_reranker_path` default | **No.** Only printed by `opendoc doctor`; `search_and_rerank` is RRF fusion, no cross-encoder runs. |
| Chat / answer generation | none by default; any OpenAI-compatible `/chat/completions` endpoint (BYOK) | `opendoc-llm` | Yes, whatever the user configures. |
| Planned local SLM | qwen2.5-3b-instruct (GGUF), bge-reranker-v2-m3 (GGUF) | `openspec/specs/task-execution-ai-engines` | Not implemented. |

Models named only in docs and UI text:
- README: "Claude 3.7 Sonnet, GPT-4o/o3-mini, Gemini 1.5 Pro, Grok 3".
- WebUI placeholder: `gpt-4o` or `claude-3-5-sonnet` (`SettingsPage.tsx:467`).
- docs-site Ollama guide: `llama3`, `mistral`, `gemma2:9b`, `qwen2:1.5b`, `phi3`.
- Tests: `deepseek-chat`, `deepseek-coder` (fixtures only).

The chat client speaks only the OpenAI protocol. The WebUI offers "Anthropic" as a provider type, but requests still go to `{base}/chat/completions`, so Claude only works through an OpenAI-compatible endpoint.

## State of the art (November 2026)

### Embeddings (multilingual retrieval)

| Model | Params / dim | Multilingual MTEB v2 | License | Runs via |
|---|---|---|---|---|
| harrier-oss-v1-27b (Microsoft, Mar/Apr 2026) | 27B / 5376 | 74.3 | MIT | HF only |
| Qwen3-Embedding-8B (Alibaba, Jun 2025) | 8B / up to 4096 | 70.58 | Apache 2.0 | Ollama `qwen3-embedding:8b` |
| harrier-oss-v1-0.6b | 0.6B / **1024** | 69.0 | MIT | HF only |
| harrier-oss-v1-270m | 270M / 640 | 66.4 (blog) | MIT | HF only |
| Qwen3-Embedding-0.6B | 0.6B / **1024** (MRL 32–1024) | 64.33 | Apache 2.0 | Ollama `qwen3-embedding:0.6b`, fastembed feature `qwen3` |
| EmbeddingGemma-300M (Google, Sep 2025) | 300M / 768 | 61.15 | Gemma terms | Ollama, fastembed |
| **BGE-M3 (current)** | 568M / 1024 | 59.56 | MIT | Ollama, fastembed |

Ollama's embedding list (fetched 2026-11-12) also shows `embeddinggemma-2` (270m–740m, updated four weeks ago); details not verified.

**Assessment:** BGE-M3 is no longer state of the art. Same-size models score 5–10 points higher on Multilingual MTEB v2. It remains a solid, well-supported baseline: MIT license, 8K context, no query prefix needed, and available everywhere (Ollama, fastembed, TEI).

### Rerankers

- **Qwen3-Reranker-0.6B/4B/8B** (Apache 2.0, 100+ languages, 32K context) is the strongest open option in current roundups.
- **bge-reranker-v2-m3** (multilingual, permissive) is the most deployed and the conservative default; it is already in fastembed (`BGERerankerV2M3`).
- jina-reranker-v3 scores higher on BEIR but is CC BY-NC (not usable commercially).
- **Assessment:** `bge-reranker-base` is two generations old and English/Chinese only. Since no reranker runs at all, the real gap is missing reranking, not the model choice.

### Chat models (BYOK, cloud)

| Vendor | Named in the project | Current (Nov 2026) |
|---|---|---|
| Anthropic | Claude 3.5 / 3.7 Sonnet | Claude 5 family: Opus 5.5, Sonnet 5.5, Haiku 5.5, Fable 5.1 |
| OpenAI | GPT-4o, o3-mini | GPT-6 generation (GPT-6.1 Terra as lower-cost tier) — aggregator sources |
| Google | Gemini 1.5 Pro | Gemini 4 (Argon, Carbon) — aggregator sources, names unconfirmed |
| xAI | Grok 3 | Grok 4.7 — aggregator source |
| DeepSeek | deepseek-chat | DeepSeek V4.1 — aggregator source |

All named cloud models are 2–4 generations behind. This is documentation drift only, because the user picks the model.

### Local chat models (Ollama)

- **Named in the project:** llama3, mistral, gemma2, qwen2:1.5b, phi3 (all 2024).
- **Current picks** (third-party guides, Nov 2026):
  - 8 GB: Gemma 4 E4B, Qwen3/3.5 4B.
  - 16 GB: Gemma 4 12B, gpt-oss-20b, Qwen3-30B-A3B.
  - Larger GPUs: Gemma 4 26B A4B.
- **Assessment:** The guide's model table is outdated. This laptop already has `gemma4:12b`, `gemma3:4b` and `qwen3:4b`.

## Gaps

1. **Embeddings have no query/document distinction.** `ByokEmbeddingProvider::embed` and `FastEmbedProvider::embed` embed queries and documents the same way. Qwen3-Embedding and harrier require a task instruction on queries; their model cards warn that leaving it out degrades quality. Switching models therefore needs a code change, not just a config change.
2. **A model switch needs a full reindex.** The vector space changes even when the dimension stays 1024 (Qwen3-Embedding-0.6B, harrier-0.6b). There is no migration or "reindex all" path beyond per-document reindex.
3. **No reranking step.** The configured reranker is dead config.
4. **Stale model names** in README, WebUI placeholder, Ollama guide and the task-execution spec.
5. **"Anthropic" provider type is misleading:** it uses the OpenAI wire protocol.

## Recommendation

- **Keep BGE-M3 as the default for now.** It works with the current code (no query prefix), and the fastembed fallback depends on it.
- **First add the query/document split** (gap 1), then offer **Qwen3-Embedding-0.6B** as the new default for new installs:
  - same 1024 dimensions;
  - +4.8 MMTEB;
  - Apache 2.0;
  - available in both Ollama and fastembed.
- harrier-0.6b scores higher (+9.4), but it is not on Ollama or in fastembed yet; watch it.
- **Add an optional cross-encoder rerank stage** with `bge-reranker-v2-m3` from fastembed. Remove `bge-reranker-base` from the defaults.
- **Update model names** in docs, UI and README, and fix or relabel the "Anthropic" option.
- Decide each point by Q&A round before implementing.

## Sources

- [microsoft/harrier-oss-v1-0.6b](https://huggingface.co/microsoft/harrier-oss-v1-0.6b), [harrier-oss-v1-27b](https://huggingface.co/microsoft/harrier-oss-v1-27b), [harrier-oss-v1-270m](https://huggingface.co/microsoft/harrier-oss-v1-270m)
- Qwen3 Embedding technical report, arXiv 2506.05176 (MMTEB scores for Qwen3-Embedding, BGE-M3, EmbeddingGemma comparison)
- [Ollama embedding models](https://ollama.com/search?c=embedding) (fetched 2026-11-12)
- [Best Embedding Models for RAG 2026 (premai)](https://www.premai.io/blog/best-embedding-models-for-rag-2026-ranked-by-mteb-score-cost-and-self-hosting/), [Embedding leaderboard 2026 (ailog)](https://app.ailog.fr/en/blog/news/embedding-models-leaderboard-2026)
- [Best Rerankers for RAG 2026 (futureagi)](https://futureagi.com/blog/best-rerankers-for-rag-2026/), [Best local reranker models 2026 (recal)](https://www.recal.so/blog/best-local-reranker-models-2026)
- [Best Ollama Models Nov 2026 (localaimaster)](https://localaimaster.com/blog/best-ollama-models), [Best local LLMs for RAG 2026 (insiderllm)](https://insiderllm.com/guides/best-local-llms-rag/)
- [Model release timeline (llmgateway)](https://llmgateway.io/timeline), [Best AI models Nov 2026 (teamday)](https://www.teamday.ai/blog/best-ai-models-2026)
- fastembed 5.17.4 source (`src/models/text_embedding.rs`, `reranking.rs`, feature `qwen3`)

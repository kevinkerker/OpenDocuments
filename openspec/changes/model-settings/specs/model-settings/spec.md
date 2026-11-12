# Spec Delta

## Purpose

Let a user choose and switch the chat and embedding models from the WebUI, use local Ollama models without typing names, and change the embedding model without breaking search.

## ADDED Requirements

### Requirement: Model settings SHALL be global and editable in the WebUI

The instance SHALL hold exactly one chat setting (provider, model) and one embedding setting (provider kind `ollama` | `openai` | `builtin`, model, dimension, optional query instruction), stored in the database and editable in the WebUI. `config.toml` SHALL only seed them on first start.

#### Scenario: Change chat model
- **WHEN** the user selects another chat model and saves
- **THEN** the next chat request uses it without a server restart

### Requirement: The WebUI SHALL list and pull Ollama models

When Ollama answers on `127.0.0.1:11434`, the WebUI SHALL list installed models, split into chat and embedding, with the embedding dimension, and SHALL offer a recommended list whose entries can be downloaded with visible progress.

#### Scenario: Pull recommended embedder
- **WHEN** the user clicks download on `qwen3-embedding:0.6b`
- **THEN** progress is shown until completion and the model then appears among installed embedding models with dimension 1024 and its query instruction prefilled

#### Scenario: Ollama not running
- **WHEN** Ollama is unreachable
- **THEN** the WebUI says so and still allows OpenAI-compatible and built-in providers

### Requirement: Queries SHALL be embedded with the configured instruction

Documents SHALL be embedded without instruction and queries with the configured instruction prepended; with no instruction both SHALL be embedded identically.

#### Scenario: Instruction-based embedder
- **WHEN** the embedding model is `qwen3-embedding:0.6b` with its preset instruction
- **THEN** search requests embed `Instruct: …\nQuery: <query>` while indexing embeds the raw chunk text

### Requirement: Switching the embedding model SHALL NOT interrupt search

Each embedding model SHALL use its own LanceDB table. Switching with indexed documents SHALL re-embed all chunks in the background from the stored chunk text; search SHALL keep using the previous model until the job completes and then switch atomically. A failed job SHALL leave the previous model active. Uploads during a job SHALL be indexed into both tables.

#### Scenario: Switch with documents
- **WHEN** 2 documents are indexed and the user switches from BGE-M3 to `qwen3-embedding:0.6b`
- **THEN** the WebUI shows job progress, search returns results throughout, and after completion search uses the new table and the old table is removed

#### Scenario: Switch on empty index
- **WHEN** no documents are indexed
- **THEN** the switch takes effect immediately without a job

### Requirement: New installs SHALL prefer Ollama

On first start with Ollama reachable, the WebUI SHALL offer `qwen3-embedding:0.6b` as embedding model and the installed chat models for chat; without Ollama the built-in BGE-M3 fallback SHALL apply.

#### Scenario: First start with Ollama
- **WHEN** a fresh install starts while Ollama runs without an embedding model
- **THEN** the server starts with the built-in fallback and the WebUI settings show a one-click "download and use qwen3-embedding:0.6b"

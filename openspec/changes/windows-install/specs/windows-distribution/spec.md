# Spec Delta

## Purpose

Make OpenDocuments installable and startable on Windows without a toolchain, ship complete release artifacts on every platform, let a fresh install start without a pre-configured provider, and keep the server local by default.

## ADDED Requirements

### Requirement: Release artifacts SHALL be complete on every platform

Every published release SHALL contain both `opendoc` and `opendoc-engine-lancedb` for each supported platform (linux-x86_64, macos-aarch64, windows-x86_64), each covered by `checksums.txt`. On Windows both executables SHALL ship in one archive `opendoc-windows-x86_64.zip`.

#### Scenario: Install script on Linux or macOS
- **WHEN** a user runs `install.sh` against a release
- **THEN** both `opendoc` and `opendoc-engine-lancedb` are installed into the same directory and checksum-verified

#### Scenario: Install script on Windows
- **WHEN** a user runs `irm <raw-url>/install.ps1 | iex`
- **THEN** both executables land in `%LOCALAPPDATA%\Programs\OpenDocuments`, that directory is on the user PATH, and `opendoc --help` works in a new terminal without administrator rights

### Requirement: The home directory SHALL resolve on Windows

Configuration and data paths SHALL resolve from `HOME`, falling back to `USERPROFILE` when `HOME` is unset or empty.

#### Scenario: Standard Windows environment
- **WHEN** `opendoc` runs with `USERPROFILE` set and `HOME` unset
- **THEN** it reads and writes `%USERPROFILE%\.config\opendocuments\config.toml` and `%USERPROFILE%\.opendocuments`

### Requirement: A fresh install SHALL start without a configured provider

When no LLM provider is configured and no documents are indexed, `opendoc start` SHALL fall back to the built-in local embedder (when compiled with `embedding-fastembed`), persist `embedding_backend = "fastembed"`, and announce the one-time model download before it starts. When documents already exist, it SHALL NOT switch embedders and SHALL fail with a message naming the config file and the `embedding_backend = "fastembed"` switch (which requires a reindex).

#### Scenario: First start
- **WHEN** a release build starts against an empty database with no provider
- **THEN** the server starts, `config.toml` contains `embedding_backend = "fastembed"`, and documents can be indexed and searched

#### Scenario: Provider lost on an existing index
- **WHEN** no provider is configured but documents are indexed
- **THEN** startup fails with a message and the embedding backend is unchanged

### Requirement: The server SHALL bind to localhost by default

`opendoc start` SHALL listen on `127.0.0.1` unless `--host` names another address.

#### Scenario: Default start
- **WHEN** a user runs `opendoc start`
- **THEN** the server is reachable on `http://127.0.0.1:3000` and not from other hosts

#### Scenario: Network exposure on request
- **WHEN** a user runs `opendoc start --host 0.0.0.0`
- **THEN** the server listens on all interfaces

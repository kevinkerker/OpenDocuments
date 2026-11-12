# Proposal

## Why

OpenDocuments lässt sich unter Windows nicht installieren (Stand 2026-11-12):

- `install.sh` und `release.yml` kennen nur Linux und macOS.
- Releases enthalten nur `opendoc`, nicht die Sidecar-Engine `opendoc-engine-lancedb`, die `opendoc` beim Start aus dem PATH startet. Auch der Einzeiler für Linux/macOS liefert damit keine lauffähige Installation.
- Das Home-Verzeichnis kam nur aus `HOME`, das ein Standard-Windows nicht setzt → Abbruch beim Laden der Konfiguration.
- Ein frisch installiertes `opendoc start` bricht ab („未設定任何 LLM provider“): Provider lassen sich nur über die API des laufenden Servers anlegen.
- Der Server bindet auf `0.0.0.0` ohne API-Key; auf einem Laptop im WLAN ist die Dokumentensuche damit im Netz offen.

Ein nativer MSVC-Build läuft ohne Codeänderung durch (VS Build Tools, `protoc`, `cmake`), es fehlt also Verteilung und Erststart, nicht Portierung.

## What Changes

- **Release:** Windows-Job (`windows-latest`, x86_64) in `release.yml`, Asset `opendoc-windows-x86_64.zip` mit `opendoc.exe` und `opendoc-engine-lancedb.exe`. Linux/macOS veröffentlichen die Engine zusätzlich als eigenes Asset. Release-Builds aktivieren `embedding-fastembed`.
- **`install.ps1`:** `irm …/install.ps1 | iex` lädt die Zip, prüft die Checksumme, entpackt nach `%LOCALAPPDATA%\Programs\OpenDocuments` und trägt den Ordner in den User-PATH ein. `install.sh` installiert die Engine mit.
- **Home-Verzeichnis:** Fallback `HOME` → `USERPROFILE`.
- **Erststart ohne Provider:** Ist kein Provider eingerichtet und noch kein Dokument indiziert, fällt `opendoc start` auf das eingebaute fastembed (BGE-M3, 1024 Dim.) zurück und schreibt `embedding_backend = "fastembed"` in die Konfiguration. Ohne das Feature oder mit vorhandenen Dokumenten bleibt der Fehler, ergänzt um einen Hinweis.
- **Bind-Adresse:** Standard `127.0.0.1`, neues Flag `opendoc start --host <ip>`.

## Capabilities

### New Capabilities

- `windows-distribution`: Installierbarkeit unter Windows, vollständige Release-Artefakte, Erststart ohne Provider, lokale Bind-Adresse.

### Modified Capabilities

- `binary-size-architecture`: Release-Distributionen bauen mit `embedding-fastembed`; Budget für dieses Feature 35 MiB statt 30 MiB (gemessen +33 MiB unter Windows). Der Default von `cargo build` bleibt ohne das Feature.

## Impact

- `.github/workflows/release.yml`, `install.sh`, neu `install.ps1`
- `crates/opendoc-cli/src/main.rs` (Start: `--host`, Fallback), `crates/opendoc-mcp/src/lib.rs` (`start_mcp_and_api_server` nimmt `SocketAddr`), `crates/opendoc-storage/src/lib.rs` (Home)
- Doku: `README.md`, `docs-site/guide/deployment.md`
- **Bricht:** Wer den Server bisher aus dem Netz erreicht hat, muss `--host 0.0.0.0` ergänzen.

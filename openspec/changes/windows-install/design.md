# Design

Stand 2026-11-12. Entscheidungen per Frage-Antwort-Spiel mit dem User getroffen; je Entscheidung die betrachteten Optionen.

## Decisions

### D1 Paketform: `install.ps1` + Release-Zip

- **Gewählt:** Windows-Job in `release.yml` packt beide `.exe` in `opendoc-windows-x86_64.zip`; `install.ps1` installiert nach `%LOCALAPPDATA%\Programs\OpenDocuments` und erweitert den User-PATH (Muster von rustup, bun, deno). Keine Toolchain auf dem Zielrechner.
- Verworfen: MSI über cargo-wix + winget (SmartScreen-Warnung ohne Code-Signing, Manifest-PR bei Microsoft), lokales Build-Skript (braucht VS Build Tools, protoc, cmake, Node), Scoop-Manifest (setzt Scoop voraus).
- Skripte, Doku und Update-Prüfung zeigen auf `kevinkerker/OpenDocuments` (Fork); `$env:OPENDOC_RELEASE_URL` überschreibt die Download-Basis.

### D2 Erststart ohne Provider: lokaler Embedder als Fallback

- **Gewählt:** Liefert der BYOK-Pfad „kein Provider“ und ist die Dokumenttabelle leer, schaltet `opendoc start` auf fastembed um und **persistiert** `embedding_backend = "fastembed"`. Persistieren verhindert, dass ein später angelegter Chat-Provider die Embeddings still in einen anderen Vektorraum wechselt; ein Wechsel bleibt eine bewusste Konfigurationsänderung samt Reindex.
- Mit vorhandenen Dokumenten kein Fallback (deren Vektoren stammen von einem anderen Embedder), stattdessen der bisherige Fehler mit Hinweis.
- `Bgem3Embedding::try_new` lädt das Modell beim Erzeugen; vor dem ersten Start wird deshalb ein Hinweis auf den einmaligen Download ausgegeben.
- Verworfen: Start mit 503-Hinweis und Nachladen (ohne Provider keine Suche), Stichwortsuche ohne Gate (reine FTS5-Treffer haben keine Cosine-Distanz und scheitern laut `hybrid-rag-retrieval` am Schwellwert; hätte Spec-Änderung gebraucht), gesperrte UI mit Einrichtungsassistent.

### D3 Spec-Konflikt fastembed: nur Release-Builds mit Feature

- `binary-size-architecture` verbietet fastembed im Default-Feature-Set und begrenzt optionale Features auf 30 MiB; gemessen +33 MiB (`opendoc.exe` 28,6 → 63,3 MB, MSVC, onnxruntime statisch).
- **Gewählt:** `cargo build` bleibt ohne Feature; Release-Artefakte und die Cargo-Fallbacks der Installskripte bauen mit `--features embedding-fastembed`. Budget für dieses Feature: 35 MiB.
- Verworfen: fastembed in die Engine verlagern (spec-treu laut `lancedb-engine-sidecar` §222, aber Protokolländerung, mehrere Tage), zwei Release-Varianten, Fallback verwerfen.

### D4 Bind-Adresse: `127.0.0.1` + `--host`

- **Gewählt:** Standard nur localhost (wie Ollama, Jupyter); `opendoc start --host 0.0.0.0` für Netzbetrieb. Erspart unter Windows auch den Firewall-Dialog beim ersten Start.
- Verworfen: `0.0.0.0` nur mit gesetztem `api_key`, unverändert lassen und dokumentieren, Feld `server.bind` in `config.toml`.

## Risks / Trade-offs

- **Download beim ersten Start** blockiert den Start ohne Fortschrittsanzeige (fastembed `show_download_progress(false)`) → Hinweiszeile vorab.
- **Releases im Fork:** Windows-Assets gibt es erst nach einem Tag-Release mit diesem Workflow; bis dahin bleibt der lokale Build (siehe `docs-site/guide/deployment.md`). Die Doku-Links auf `kevinkerker.github.io` setzen aktivierte GitHub Pages voraus.
- **Konsolenfenster der Engine:** `lancedb-engine-sidecar` verlangt unter Windows Start ohne Konsolenfenster und Kill-on-parent-close; `sidecar_client.rs` setzt keine `creation_flags`. Nicht Teil dieses Changes, offen.

## Migration

- Bestehende Installationen mit Provider: unverändert, außer Bind-Adresse (D4).
- Bestehende Installationen ohne Provider, aber mit Dokumenten: weiterhin Fehler, jetzt mit Hinweis.

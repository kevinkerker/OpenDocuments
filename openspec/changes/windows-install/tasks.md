# Tasks

## 1. Laufzeit

- [x] 1.1 Home-Fallback `HOME` → `USERPROFILE` (`opendoc-storage`, `install-opencode`)
  - Prüfung: Unit-Test `home_dir_falls_back_to_userprofile`; Start ohne `HOME` legt Konfiguration unter `USERPROFILE` an
- [x] 1.2 `opendoc start --host`, Standard `127.0.0.1`
  - Prüfung: Unit-Test `start_binds_localhost_by_default`
- [x] 1.3 fastembed-Fallback beim Erststart (leere Dokumenttabelle, kein Provider), Persistenz in `config.toml`, Hinweis vor Download
  - Prüfung: Unit-Test `fastembed_fallback_only_on_empty_index`; Rauchtest 2026-11-12: Erststart 65 s (Modell 560 MB), Config auf `fastembed`, Upload 14 Chunks, Suche HTTP 200 mit 3 Treffern, Neustart 4 s

## 2. Verteilung

- [x] 2.1 `release.yml`: Windows-Job, Engine-Assets für alle Plattformen, `--features embedding-fastembed`
  - Nicht lokal prüfbar (kein YAML-Parser, kein Runner); erster Tag-Build ist die Prüfung
- [x] 2.2 `install.ps1` (Download, Checksumme, Entpacken, User-PATH)
  - Prüfung: `irm | iex` gegen lokale Fake-Release (Zip + checksums.txt im `*`-Format von Git-Bash-sha256sum): Installation, PATH als `ExpandString` ohne Duplikat, `opendoc --help` über PATH, falsche Checksumme bricht ab
- [x] 2.3 `install.sh`: Engine mitinstallieren, Cargo-Fallback inkl. Engine und Feature
  - Prüfung: `sh -n`, Checksum-Grep gegen Beispielzeilen; Download-Pfad nicht ausgeführt (kein Release mit Engine-Asset)

## 3. Doku

- [x] 3.1 README und `docs-site/guide/deployment.md`: Windows-Installation, `--host`, Erststart
- [ ] 3.2 `binary-size-architecture`: Budget-Zeile fastembed beim Archivieren anpassen (35 MiB, nur Release)

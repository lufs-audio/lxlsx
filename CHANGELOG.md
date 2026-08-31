# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-31

### Added

- `lxlsx` CLI: `snapshot`, `run`, `list`, `verify`, `index`.
- Kernel: typed `Snapshot` document, SQLite reader (v2 schema), xlsx renderer (six-block golden layout), read-back verifier.
- Process #1 (`snapshot`): the budget review control panel.
- Filesystem-as-registry (`processes/` + generated `index.json`).
- Golden-parity integration tests against an in-memory fixture.
- House exit-code floor (`0/2/5`) and JSON envelope.

# Changelog

All notable changes to `cgrade` are documented here. The project follows
Semantic Versioning.

## 0.1.2 - 2026-09-25

- `jsr.json` declares the bare SPDX id `LGPL-3.0-only`; JSR rejected the
  `WITH` expression. The linking exception still ships in `LICENSES/`.
- 0.1.1 reached crates.io only; npm, JSR and PyPI go from 0.1.0 to 0.1.2.

## 0.1.1 - 2026-09-24

- Repository moved to github.com/corbet-foss/cgrade; registry metadata points there.
- Released from a single tag through CI (crates.io and JSR trusted publishing).
- Drop the duplicate `LICENSES/LGPL-3.0-only WITH LGPL-3.0-linking-exception.txt`
  (identical to `LGPL-3.0-linking-exception.txt`); JSR rejects paths with spaces.

## 0.1.0 - 2026-09-18

- `parse_grade` (dot OR comma input, strict) and `format_grade` (explicit
  decimals, dot separator always: `"6.0"`, never `"6,0"`), integer-exact
  in every port.
- Initial release: seven country grading scales (`at`, `ch`, `de`, `es`,
  `fr`, `it`, `us`) with best/worst/pass bounds plus the official
  modified Bavarian formula (KMK Beschluss) for foreign-to-German
  conversion, truncated — never rounded — to 1 decimal (2 decimals for
  the inverse), as a Rust crate, a pure-TypeScript package, a
  pure-Python package with JSON CLI, and a Typst module sharing one
  table and one vector suite (74 vectors). Invalid systems and grades
  yield no output. Data only: no grade labels.

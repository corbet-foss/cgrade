# Changelog

All notable changes to `cgrade` are documented here. The project follows
Semantic Versioning.

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

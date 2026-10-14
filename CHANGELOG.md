# Changelog

## 0.1.0 (2026-10-14)

First release.

- Levels `Balanced`, `High`, `Max`: DEFLATE competition submissions #521, #85 and #243,
  unmodified, with their Lean proofs in `parsers/`.
- gzip, zlib, raw DEFLATE and BGZF writers (`GzEncoder`, `ZlibEncoder`, `DeflateEncoder`,
  `bgzf::BgzfEncoder`) and one-shot functions; streaming in 1 MiB chunks with a 32 KiB
  history; adaptive block splitting.
- `vgzip`, a gzip-compatible command-line tool (feature `cli`, on by default).
- Output is byte-identical to the competition's benchmark harness for every level and
  format (`tests/golden.rs`).

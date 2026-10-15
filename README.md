# verified-deflate

gzip, zlib, raw DEFLATE and BGZF compression whose LZ77 parsers are **proved correct
in Lean**: for every possible input, the tokens the parser emits decode back to that
input.

The parsers are the best general-purpose submissions to the
[conjectures.io DEFLATE competition](https://conjectures.io/competitions/deflate)
(28 September – 12 October 2026), where miners wrote LZ77 parsers in Rust together with
machine-checked proofs of their correctness. Of 648 accepted submissions, these three
held up best on real data none of them had been tuned on.

The output is ordinary DEFLATE. gunzip, zlib, browsers, Python's `gzip`, `unzip` and
samtools read it unchanged; nothing new is needed to decompress.

## Install

```bash
cargo install verified-deflate        # the `vgzip` command
```

```toml
[dependencies]
verified-deflate = { version = "0.1", default-features = false }   # library only
```

## Command line

```bash
vgzip -9 site.js                       # site.js -> site.js.gz, like gzip
vgzip -k --best *.css *.html           # keep the originals
tar c logs/ | vgzip > logs.tar.gz      # stdin to stdout
vgzip --bgzf -c calls.vcf > calls.vcf.gz   # BGZF, for samtools and tabix
vgzip -d site.js.gz                    # or plain gunzip
```

Levels follow gzip: `-1`…`-3` (or `--fast`) Balanced, `-4`…`-6` High (default),
`-7`…`-9` (or `--best`) Max. `-c`, `-k`, `-f`, `-d` and `-v` behave like gzip's.

## Library

```rust
use std::io::Write;
use verified_deflate::{GzEncoder, Level};

let mut gz = GzEncoder::new(std::fs::File::create("out.gz")?, Level::Max);
std::io::copy(&mut std::fs::File::open("in.txt")?, &mut gz)?;
gz.finish()?;
```

- Writers: `GzEncoder`, `ZlibEncoder`, `DeflateEncoder` (raw) and `bgzf::BgzfEncoder`,
  all `std::io::Write`. `flush()` is a sync flush (zlib's `Z_SYNC_FLUSH`). Dropping a
  writer finishes it; call `finish()` to see errors.
- One-shot: `gzip(&data, level)`, `zlib(…)`, `deflate(…)`, `bgzf::compress(…)`.
- `Options` sets the chunk size and the block mode.
- `verified_deflate::parsers` exposes the proved parsers for anyone who wants raw LZ77
  tokens.

Dependencies of the library: `crc32fast` and `adler2`. The `cli` feature (on by
default) adds `flate2` for `vgzip -d`.

## Decompressing

This crate only compresses. The output is standard, so use any inflater, e.g.
[`flate2`](https://crates.io/crates/flate2) or [`miniz_oxide`](https://crates.io/crates/miniz_oxide):

```toml
[dependencies]
flate2 = "1"        # streams: gzip, zlib, BGZF
miniz_oxide = "0.8" # in-memory buffers
```

```rust
use std::io::Read;

// gzip or BGZF (BGZF is multi-member gzip), streaming from any reader:
let mut data = Vec::new();
flate2::read::MultiGzDecoder::new(std::fs::File::open("out.gz")?).read_to_end(&mut data)?;

// zlib or raw DEFLATE, in memory:
let zlib_bytes = std::fs::read("out.zz")?;
let data = miniz_oxide::inflate::decompress_to_vec_zlib(&zlib_bytes).expect("valid zlib");
let raw_bytes = std::fs::read("out.deflate")?;
let data = miniz_oxide::inflate::decompress_to_vec(&raw_bytes).expect("valid DEFLATE");
```

On the command line: `gunzip`, `zcat`, `vgzip -d`, or `samtools`/`tabix` for BGZF.

## Levels and what to expect

| level | parser | large files¹ | web assets² | logs³ | speed⁴ |
|---|---|---|---|---|---|
| `Balanced` | competition #521 | 4.3% smaller than zlib -6, 3.5% smaller than `gzip -9` | 1.5% smaller than zlib -6 | 2.5% larger than zlib -6 | ≈ 27 MB/s |
| `High` (default) | competition #85 | 4.5% / 3.7% smaller | 2.1% smaller | 14% smaller | ≈ 16 MB/s |
| `Max` | competition #243 | 5.0% / 4.1% smaller; smaller than 7-Zip's deflate | 2.5% smaller: libdeflate level 12's size, zopfli within 0.5% | 13% smaller | ≈ 6 MB/s |

On the same machine: zlib -6 ≈ 39 MB/s, `gzip -9` ≈ 17 MB/s, zlib-ng -6 ≈ 80 MB/s,
libdeflate -12 ≈ 5 MB/s, zopfli ≈ 0.2 MB/s.

¹ 824 MB: the Silesia corpus, enwik8, CPython and Linux source tarballs, GitHub event
data; streamed in 1 MiB chunks. ² 345 MB, 13,268 files from npm packages and the Python
documentation. ³ 281 MB of system, web-server and application logs with a sync flush
every 64 KiB. ⁴ One core of an AMD EPYC 9354 virtual machine. Sizes are geometric means
over files. `tests/golden.rs` checks that this crate writes exactly the bytes the
competition's benchmark harness measured.

**Good for**: precompressing static web assets (nginx `gzip_static`, CDNs), release
tarballs, backups and archives, rotated logs, BGZF genomics files, ZIP and JAR members:
anywhere output size matters more than speed and data is compressed once and read many
times.

**Not good for**: on-the-fly HTTP compression or log shipping where throughput matters
(zlib-ng is 3–14× faster), PNG image data (oxipng and zopflipng are 4–6% smaller), or
many tiny messages each flushed separately: every flush re-parses up to 32 KiB of
history, which makes that case 7–45× slower than zlib.

## What is proved, and what is tested

- **Proved** in Lean 4 and checked by the competition's validator: each parser
  terminates, never fails, stays within its buffers, and its tokens decode back to the
  input, for every input.
- **Tested, not proved**: everything after the parse. That covers Huffman coding and block
  splitting (`src/encoder.rs`), the chunking that lets a one-shot parser stream
  (`src/stream.rs`), and the gzip, zlib and BGZF framing. Every test output is decoded by
  miniz_oxide, zlib and libdeflate; the benchmark harness that shares this code checked
  about 2 GB of real data and 4,767 randomised streaming cases the same way.
- **Defence in depth**: the encoder rejects an illegal token or a token stream that does
  not cover the input, so a violated contract becomes an `io::Error`, never corrupt
  output.
- **Not claimed**: anything about ratio or speed beyond the measurements above. The
  proofs say nothing about either.

## Behaviour

- **Streaming**: input is compressed in 1 MiB chunks, each parsed with the previous
  32 KiB as history, so the output is one continuous stream with full-window matches.
- **Memory**: peak about 26 MiB (Balanced), 21 MiB (High), 43 MiB (Max), the same for a
  5 MB and a 108 MB input.
- **Determinism**: the same input and options always give the same bytes, however the
  input is split across `write` calls. gzip headers carry no file name or time.

## Where the code comes from

| part | files | written by |
|---|---|---|
| LZ77 parsers and their Lean proofs | `parsers/p521`, `parsers/p85`, `parsers/p243` | the competition's miners (table below), unmodified |
| Huffman coding and block writing | `src/encoder.rs` | the competition's shared encoder, written for the competition by its operator; made incremental for this crate |
| Adaptive block splitting | `src/encoder.rs` (`split_ranges`) | written for this crate; the idea (cut where symbol statistics change, keep a cut only if the exact cost drops) is the one zopfli's block splitter popularised |
| Token format | `src/token.rs` | the competition's, unchanged |
| Streaming, gzip/zlib/BGZF framing, `vgzip` | `src/stream.rs`, `src/lib.rs`, `src/bgzf.rs`, `src/bin/vgzip.rs` | written for this crate; the API's shape (`GzEncoder`, `finish`, `try_finish`) follows flate2's so it feels familiar |

The formats follow their specifications: RFC 1951 (DEFLATE), RFC 1950 (zlib),
RFC 1952 (gzip) and the SAM/BAM specification §4.1 (BGZF). The lookup tables they imply
(length and distance codes, extra bits, the code-length order) are the same in every
DEFLATE implementation.

**How this was checked.** Every source file here was compared with miniz_oxide 0.8.9,
zopfli 0.8.4, flate2 1.1.10, libdeflate 1.26 and zlib 1.3.2 (12-token runs with comments
removed, both verbatim and with identifiers renamed), and with all of the competition's
example parsers. The parsers and the encoder share at most about 1% with any of those
libraries, and only the RFC lookup tables above; the parsers also contain none of those
libraries' characteristic identifiers (`nice_match`, `max_chain`, `hc_matchfinder`,
`SymbolStats`, …). The writers in `src/lib.rs` and `src/bgzf.rs` share 6–11% of their
token runs with flate2's `write.rs`: `std::io::Write` method signatures and the method
names this API mirrors on purpose; the bodies are different. #521 and #243 share at
most 0.5% with the competition's examples; #85 builds on the competition's `optimal-iter`
example (4.5% shared), which the competition's miner guide invites miners to do. One
competition example, `mo-lazy`, is an acknowledged reimplementation of miniz_oxide's
level-9 strategy; none of these parsers uses code specific to it.

### The parsers

`parsers/p<id>/parse.rs` is each submission's source, byte for byte, and
`parsers/p<id>/Parse.lean` the proof the validator accepted. Each `parse.rs` hashes to
the `source_sha256` the competition recorded, and both files equal what
`https://conjectures.io/v1/competitions/deflate/submissions/<id>/source` serves.

| level | submission | submitted (UTC) | author (hotkey) | `parse.rs` sha256 |
|---|---|---|---|---|
| Balanced | #521 | 2026-10-08 | `5DtLFqKpPzwUQSJ8s9NsL4K7qevfivohCx2VPASnS4hK3dxT` | `32d1fd452496892d44b8c733c2432081a31a86b517278e4769b9c8a9e07b57ef` |
| High | #85 | 2026-09-29 | `5Dvu6sE2qVL6TD4sYmEGv9rAan5GXsV7MMef32HMm67viwxP` | `97d60f0c53517c4584cbccfed5e5832afde600bf15b5ed8554377812462e51c3` |
| Max | #243 | 2026-10-02 | `5E1ih2hfMg5x6HGMQsRZ4gerXmq4Hu5S9bmG1ZLcBLyezJWH` | `a83ed10bca126a6b49ea9ed0d43d85eecef064ba97c25e60cea11c02481ee9f2` |

To check a proof yourself, use the competition repository
([conjectures-optimisation-deflate](https://github.com/conjectures-io/conjectures-optimisation-deflate))
with its toolchain set up (`./setup.sh`, about 15 minutes and 9 GB):

```bash
cp -r /path/to/verified-deflate/parsers/p243 my-check
just check-proof my-check
```

Why these three: they pass the competition evaluation's robustness screen at every input
size tried (up to 268 MB in one buffer), and #521 and #85 pass even its strictest version.
The evaluation's method, data and full results are in that repository's
`docs/wrapup/` (branch `wrapup/real-world-eval`).

## Licence

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. See [NOTICE](NOTICE) for the competition submissions and their authors.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.

## Development

```bash
cargo test --release    # round trips through three decoders, golden hashes
cargo build --release   # target/release/vgzip
```

//! Deterministic test inputs, generated rather than read from files so the tests do
//! not depend on anything outside the crate. The golden hashes in `tests/golden.rs`
//! were computed from these exact bytes by the competition's benchmark harness.

pub fn lcg(s: &mut u64) -> u64 {
    *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *s >> 33
}

/// English-like text: what most compressors see most of the time.
pub fn prose(seed: u64, len: usize) -> Vec<u8> {
    const WORDS: &[&str] = &[
        "the", "of", "and", "to", "in", "is", "was", "for", "on", "that", "with", "as", "by",
        "at", "from", "his", "her", "an", "were", "which", "this", "be", "compression", "window",
        "match", "literal", "distance", "length", "block", "stream", "token", "decode", "proof",
        "verify", "parser", "corpus", "frontier", "deflate", "gzip", "archive",
    ];
    let mut s = seed;
    let mut out = Vec::with_capacity(len + 64);
    while out.len() < len {
        let n = 4 + (lcg(&mut s) % 14) as usize;
        for i in 0..n {
            let w = WORDS[(lcg(&mut s) % WORDS.len() as u64) as usize];
            if i == 0 {
                out.extend(w[..1].to_uppercase().bytes());
                out.extend_from_slice(&w.as_bytes()[1..]);
            } else {
                out.push(b' ');
                out.extend_from_slice(w.as_bytes());
            }
        }
        out.extend_from_slice(if lcg(&mut s) % 7 == 0 { b".\n\n" } else { b". " });
    }
    out.truncate(len);
    out
}

/// Log lines: timestamps, levels, repeated templates with varying fields.
pub fn logs(seed: u64, len: usize) -> Vec<u8> {
    const MSG: &[&str] = &[
        "request completed status=200 bytes=",
        "cache miss for key user:",
        "connection reset by peer addr=10.0.",
        "worker started pid=",
        "verification finished submission=",
    ];
    let mut s = seed;
    let mut out = Vec::with_capacity(len + 128);
    let mut t = 1_791_900_000u64;
    while out.len() < len {
        t += lcg(&mut s) % 3;
        let lvl = ["INFO", "INFO", "INFO", "WARN", "DEBUG"][(lcg(&mut s) % 5) as usize];
        let m = MSG[(lcg(&mut s) % MSG.len() as u64) as usize];
        out.extend(format!("2026-10-12T{:02}:{:02}:{:02}Z {lvl} {m}{}\n", (t / 3600) % 24, (t / 60) % 60, t % 60, lcg(&mut s) % 100_000).bytes());
    }
    out.truncate(len);
    out
}

/// JSON records, as an API or a data dump emits them.
pub fn json(seed: u64, len: usize) -> Vec<u8> {
    let mut s = seed;
    let mut out = Vec::with_capacity(len + 256);
    let mut id = 1000u64;
    while out.len() < len {
        id += 1 + lcg(&mut s) % 4;
        out.extend(
            format!(
                "{{\"id\":{id},\"name\":\"user{}\",\"active\":{},\"score\":{}.{},\"tags\":[\"a{}\",\"b{}\"]}}\n",
                lcg(&mut s) % 5000,
                lcg(&mut s) % 2 == 0,
                lcg(&mut s) % 100,
                lcg(&mut s) % 100,
                lcg(&mut s) % 9,
                lcg(&mut s) % 9
            )
            .bytes(),
        );
    }
    out.truncate(len);
    out
}

pub fn random(seed: u64, len: usize) -> Vec<u8> {
    let mut s = seed;
    (0..len).map(|_| lcg(&mut s) as u8).collect()
}

/// A mix with long runs, repeats beyond the 32 KiB window and binary tables.
pub fn mixed(seed: u64, len: usize) -> Vec<u8> {
    let mut s = seed;
    let mut out = Vec::with_capacity(len + 70_000);
    while out.len() < len {
        match lcg(&mut s) % 5 {
            0 => out.extend(prose(lcg(&mut s), 2000 + (lcg(&mut s) % 20_000) as usize)),
            1 => out.extend(std::iter::repeat(lcg(&mut s) as u8).take((lcg(&mut s) % 5000) as usize)),
            2 => {
                let mut v = lcg(&mut s) as u32;
                for _ in 0..(lcg(&mut s) % 4000) {
                    v = v.wrapping_add(1 + (lcg(&mut s) % 3) as u32);
                    out.extend_from_slice(&v.to_le_bytes());
                }
            }
            3 if out.len() > 40_000 => {
                // A copy from 33-40 KiB back: just out of reach of the window.
                let from = out.len() - 33_000 - (lcg(&mut s) % 7000) as usize;
                let n = (lcg(&mut s) % 3000) as usize;
                let piece = out[from..from + n].to_vec();
                out.extend(piece);
            }
            _ => out.extend(random(lcg(&mut s), (lcg(&mut s) % 3000) as usize)),
        }
    }
    out.truncate(len);
    out
}

/// The named inputs every test uses.
pub fn all() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("empty", Vec::new()),
        ("one-byte", b"a".to_vec()),
        ("tiny", b"hello hello hello hello".to_vec()),
        ("prose-200k", prose(1, 200_000)),
        ("logs-600k", logs(2, 600_000)),
        ("json-300k", json(3, 300_000)),
        ("random-100k", random(4, 100_000)),
        ("mixed-2600k", mixed(5, 2_600_000)),
    ]
}

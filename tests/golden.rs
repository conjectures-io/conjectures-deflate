//! The bytes this crate writes are the bytes the competition's benchmark harness
//! measured: the published sizes and speeds describe this code exactly.

mod common;
use sha2::{Digest, Sha256};
use verified_deflate::{bgzf, deflate, Level, Options};

fn hex(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn output_matches_the_benchmark_harness() {
    let inputs = common::inputs::all();
    let mut checked = 0;
    for &(level, input, kind, want) in common::golden_values::GOLDEN {
        let level = match level {
            "Balanced" => Level::Balanced,
            "High" => Level::High,
            "Max" => Level::Max,
            l => panic!("unknown level {l}"),
        };
        let data = &inputs.iter().find(|(n, _)| *n == input).expect(input).1;
        let got = match kind {
            "raw" => deflate(data, level),
            "raw-fixed-blocks" => deflate(data, Options { level, adaptive_blocks: false, ..Options::default() }),
            "bgzf" => bgzf::compress(data, level),
            k => panic!("unknown kind {k}"),
        };
        assert_eq!(hex(&got), want, "{level:?} {input} {kind}");
        checked += 1;
    }
    assert_eq!(checked, 3 * inputs.len() * 3);
}

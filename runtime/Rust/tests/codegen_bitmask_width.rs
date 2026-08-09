//! Regression guard for the 32-bit bitset-shift bug (antlr4rust/antlr4#47).
//!
//! `bitsetBitfieldComparison` in Rust.stg must emit `1u64 <<`, never
//! `1usize <<`: the accompanying `testShiftInRange` guard admits shift
//! amounts 0..63 (64-bit word semantics), and on 32-bit targets
//! (wasm32, i686) a usize shift wraps mod 32 -- `1usize << 35`
//! evaluates to `1 << 3` -- producing false token-set membership and
//! wrong ATN predictions. The same generated code behaves correctly on
//! 64-bit hosts, which is why this is enforced by scanning the
//! generated sources rather than by a parse test: a behavioral repro
//! only fails when the test suite itself runs on a 32-bit target
//! (e.g. `cargo test --target wasm32-wasip1` under a WASM runtime).
//!
//! `tests/gen` is regenerated from the grammars by build.rs whenever
//! the tool jar is present, so this catches a template regression on
//! any CI run that builds the tool first.

use std::fs;

#[test]
fn generated_bitset_shifts_are_u64() {
    let mut offenders = Vec::new();
    for entry in fs::read_dir("tests/gen").expect("tests/gen exists") {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        if source.contains("1usize <<") {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "generated parsers use `1usize <<` in bitset comparisons, which \
         mis-parses on 32-bit targets (see issue #47); Rust.stg's \
         bitsetBitfieldComparison must emit `1u64 <<`. Offending files: \
         {offenders:?}"
    );
}

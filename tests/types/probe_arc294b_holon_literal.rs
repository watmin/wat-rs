//! Arc 294.b — DISCONFIRMING PROBE for the `#holon` relaxed EDN literal (the clj↔wat seam).
//!
//! Decision (`294/NOTE-holon-literal-tag.md`, four-questions-selected): a `#holon`-tagged literal types as a
//! **heterogeneous `Hologram`** (any EDN — disparate keys AND values), NOT a monomorphic `HashMap<K,V>`. You
//! declare what it IS (holon/EDN), not what it holds. Same bytes read as a Hologram in wat and as plain
//! identity-data in Clojure (a one-line `holon → identity` data-reader) — the byte-identical bridge.
//!
//! THE WAT SOURCE IS A REAL FILE — [`wat-scripts/demos/holon-literal/cosine.wat`] — slurped here at test time,
//! not inlined. So the *same bytes* the Rust probe measures are the bytes a Clojure consumer (or `cargo wat`)
//! reads: the fixture IS the 294.b showpiece (one source on disk, two readers). Slurp-a-wat precedent:
//! `tests/nursery/probe_arc214_stone81b_*.rs`.
//!
//! RED at HEAD: the wat SOURCE reader (`crates/wat-reader`) has no `#tag <form>` dispatch — only `#{` (set,
//! `lexer.rs:318`). So `#holon {…}` parses as TWO forms (`#holon` + `{…}`), making `(cosine #holon {…} #holon {…})`
//! a 4-arg call (`ArityMismatch { expected: 2, got: 4 }`), and the heterogeneous map separately trips monomorphic
//! literal inference. GREEN when `#holon` reads as ONE tagged literal that types as `Hologram` — so a
//! *heterogeneous* map (which `infer_map_literal` rejects monomorphically) measures directly (identical → ~1.0).

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// The 294.b demo fixture — slurped, not inlined; the same bytes a Clojure data-reader will read.
const DEMO_FIXTURE: &str = "wat-scripts/demos/holon-literal/cosine.wat";

/// `#holon {heterogeneous}` reads as one Hologram literal and measures — heterogeneous keys AND values.
#[test]
fn holon_tag_makes_heterogeneous_edn_measure() {
    let src = std::fs::read_to_string(DEMO_FIXTURE)
        .unwrap_or_else(|e| panic!("294.b demo fixture {DEMO_FIXTURE} must exist (run from crate root): {e}"));
    // GREEN TARGET: `#holon {…}` is ONE Hologram literal; the heterogeneous map types as Hologram (not a
    //   monomorphic HashMap), so cosine gets 2 args and measures (cosine of identical → ~1.0).
    // RED AT HEAD: `#holon` + `{…}` parse as separate forms → cosine gets 4 args → ArityMismatch at check.
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "#holon should make a heterogeneous EDN literal type as a Hologram and measure; got: {:?}",
        world.err()
    );
}

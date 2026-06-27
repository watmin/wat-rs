//! Arc 294.b — DISCONFIRMING PROBE for the `#holon` relaxed EDN literal (the clj↔wat seam).
//!
//! Decision (`294/NOTE-holon-literal-tag.md`, four-questions-selected): a `#holon`-tagged literal types as a
//! **heterogeneous `Hologram`** (any EDN — disparate keys AND values), NOT a monomorphic `HashMap<K,V>`. You
//! declare what it IS (holon/EDN), not what it holds. Same bytes read as a Hologram in wat and as plain
//! identity-data in Clojure (a one-line `holon → identity` data-reader) — the byte-identical bridge.
//!
//! RED at HEAD: the wat SOURCE reader (`crates/wat-reader`) has no `#tag <form>` dispatch — only `#{` (set,
//! `lexer.rs:318`). So `#holon {…}` parses as TWO forms (`#holon` + `{…}`), making `(cosine #holon {…} #holon {…})`
//! a 4-arg call (proven manually this session). GREEN when `#holon` reads as ONE tagged literal that types as
//! `Hologram` — so a *heterogeneous* map (which `infer_map_literal` rejects monomorphically) measures directly.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// `#holon {heterogeneous}` reads as one Hologram literal and measures — heterogeneous keys AND values.
#[test]
#[ignore = "RED at HEAD: arc-294.b #holon literal not built (reader has no #tag dispatch); un-ignore when #holon types as Hologram"]
fn holon_tag_makes_heterogeneous_edn_measure() {
    let src = r#"
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::do
            (:wat::kernel::pprintln
              (:wat::holon::cosine
                #holon {:kw ["a" "b"] true #{1 :foo "bar"} 3.0}
                #holon {:kw ["a" "b"] true #{1 :foo "bar"} 3.0}))
            nil))
    "#;
    // GREEN TARGET: `#holon {…}` is ONE Hologram literal; the heterogeneous map types as Hologram (not a
    //   monomorphic HashMap), so cosine gets 2 args and measures (cosine of identical → ~1.0).
    // RED AT HEAD: `#holon` + `{…}` parse as separate forms → cosine gets 4 args → ArityMismatch at check.
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "#holon should make a heterogeneous EDN literal type as a Hologram and measure; got: {:?}",
        world.err()
    );
}

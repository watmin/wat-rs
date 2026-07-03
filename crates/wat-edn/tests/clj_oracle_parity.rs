//! THE clj-ORACLE DIFFERENTIAL WARD — `clojure.edn` is the oracle; non-parity is an illegal state.
//!
//! For every input in the corpus, wat-edn's accept/refuse must match `clojure.edn`'s — EXCEPT the
//! explicit, justified exemptions below. The oracle's verdicts are baked in `clj_oracle/golden.txt`
//! (so this ward runs WITHOUT clojure); regenerate them via `clj_oracle/regen.clj` (needs the
//! `clojure` CLI) whenever `clj_oracle/corpus.txt` grows:
//!
//!   CORPUS=crates/wat-edn/tests/clj_oracle/corpus.txt \
//!   GOLDEN=crates/wat-edn/tests/clj_oracle/golden.txt \
//!     clojure -M crates/wat-edn/tests/clj_oracle/regen.clj
//!
//! Grow the corpus until it stops finding divergences (loop-until-dry). The obligation is directional:
//! **wat must accept everything clj accepts** — a `clj:OK / wat:ERR` row is a wat bug. A `clj:ERR /
//! wat:OK` row means wat read *valid* EDN clj's default declined (e.g. an unknown tag read generically):
//! examine it — a wat *superset* of valid EDN is allowed (exempt it with a reason), accepting *invalid*
//! EDN is a bug.

const GOLDEN: &str = include_str!("clj_oracle/golden.txt");

/// The ONLY allowed divergences from the clj oracle — each with a load-bearing reason. Anything not
/// listed here must match clj exactly.
fn exemption(input: &str) -> Option<&'static str> {
    match input {
        // clj:OK / wat:ERR — needs a value type wat doesn't have yet.
        "1/2" | "-3/4" => Some(
            "ratio — wat has no rational value type yet (deferred; \
             docs/arc/2026/04/109-kill-std/NOTE-rational-number-support.md)",
        ),
        // clj:ERR / wat:OK — wat reads a VALID tagged element generically (the EDN spec's
        // 'read any and all edn' option, arc 296); clj.edn's default handler declines unknown tags.
        // Intentional wat superset, not a bug.
        "#myapp/Foo {:x 1}" => Some(
            "unknown tag — wat reads it generically (spec-blessed 'read any and all edn'); \
             clj.edn's default declines. Intentional wat superset.",
        ),
        _ => None,
    }
}

#[test]
fn wat_edn_matches_clj_oracle() {
    let mut fails = Vec::new();
    for line in GOLDEN.lines() {
        if line.is_empty() {
            continue;
        }
        let (clj, input) = line.split_once('\t').expect("golden row must be VERDICT\\tINPUT");
        let wat = match std::panic::catch_unwind(|| wat_edn::parse_owned(input)) {
            Ok(Ok(_)) => "OK",
            Ok(Err(_)) => "ERR",
            Err(_) => "PANIC",
        };
        if wat == clj {
            continue;
        }
        // A divergence — allowed only if explicitly exempted (and never a panic).
        if exemption(input).is_some() {
            assert_ne!(
                wat, "PANIC",
                "exempted input {input:?} PANICKED — an exemption may diverge on OK/ERR, never panic"
            );
            continue;
        }
        fails.push(format!("  {input:?}\tclj:{clj}\twat:{wat}"));
    }
    assert!(
        fails.is_empty(),
        "\n\nclj-oracle parity VIOLATED — wat-edn diverges from clojure.edn on {} input(s) \
         (non-parity is an illegal state):\n{}\n\nFix wat-edn to match clj, or add a justified exemption.\n",
        fails.len(),
        fails.join("\n"),
    );
}

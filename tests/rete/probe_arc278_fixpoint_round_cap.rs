//! THE CASCADE FIXPOINT TERMINATES, OR SAYS SO — it does not kill the process.
//!
//! **This is the stone `DESIGN-STONE-4b-cascade-fixpoint` deferred, and it named the shape
//! exactly.** That stone does not claim divergence is impossible — its § Termination says: *"a
//! rule that derives an unbounded stream of distinct facts (e.g. arithmetic in a fact-arg
//! producing X(n) → X(n+1)) would not terminate ... if one is ever needed, a depth/round safety
//! cap is its own future stone (let need reveal)."* The need revealed: `N(k) :- N(k-1)` mints a
//! structurally novel fact every round, so the dedup that bounds the fixpoint never bites.
//!
//! Measured 2026-08-27, in 11 lines of legal wat, before this cap existed:
//!
//! ```text
//! "firing..."
//! memory allocation of 545259536 bytes failed
//! ```
//!
//! No wat error. No span. No rule named. The PROCESS died, and with no `ulimit` that is the
//! machine's memory rather than one test's. Found by `circumspicere` as an L2
//! (`NEXT-STRIKES-theater-hunt.md`: *"Nothing protects an embedder"*), carried on
//! `RETE-OPEN-WORK.md` § 3.1, and proven live rather than argued.
//!
//! ## The two fixtures are ONE `:where` apart, and that is the test
//!
//! - `probe_arc278_fixpoint_round_cap.wat` — no guard. Must be REFUSED, with the located
//!   `#wat.runtime/FixpointRoundCapExceeded` naming the cap.
//! - `probe_arc278_fixpoint_round_cap_deep.wat` — the same rule plus `(where (< ?k 500))`. Runs
//!   **500 rounds**, ten times the deepest axis in the grid (`deep-cascade` at depth 50), and must
//!   SUCCEED with 501 facts.
//!
//! A cap that failed the second is capping DEPTH, which is a legitimate workload shape. Without
//! that row this file would only prove the engine can refuse things, not that it refuses the right
//! ones — the vacuous-gate shape this arc keeps pulling out.
//!
//! ## What is deliberately NOT claimed
//!
//! The cap bounds NON-TERMINATION, not memory. A single round may still derive without bound —
//! `fanout` derives 40_000 facts in one round — and that is not limited here. And the `$oracle`
//! carries no cap and still hangs on the first fixture: the asymmetry is bounded (the cap fires
//! only on rule sets already broken, no fuzzer can generate one without hanging its own suite,
//! and `$oracle` is the reference an embedder never runs).

use std::path::Path;
use std::process::{Command, Stdio};
use wat_edn::{Keyword, OwnedValue, Tag};

fn run(rel: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let out = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The refusal, PARSED — tag pinned, named fields read exactly.
///
/// Not `contains`: `tests/lint/no_loose_string_assert.rs` refuses a substring check where an exact
/// one belongs, and it is right here. This error is deterministic EDN, so a `contains("10000")`
/// would pass on a reordered map, on appended garbage, and — worst — on the cap appearing in some
/// unrelated field. The same call `validate.rs`'s own error tests make (*"a SUBSTRING search over
/// it is a loose check the value does not deserve"*). The lint offers a per-site rune exemption;
/// taking it would have been the easy road to a weaker test.
///
/// The wire shape is nested: the outer `LociDiedError/RuntimeError` carries a VECTOR OF STRINGS,
/// each string being the EDN text of one error, so it is parsed twice.
fn fixpoint_error(stderr: &str) -> Vec<(OwnedValue, OwnedValue)> {
    let outer = wat_edn::parse_owned(stderr.trim())
        .unwrap_or_else(|e| panic!("the refusal must be EDN on stderr; got {stderr:?} ({e})"));
    let inner_text = match outer {
        OwnedValue::Vector(mut xs) if !xs.is_empty() => match xs.remove(0) {
            OwnedValue::Tagged(_, body) => match *body {
                OwnedValue::Vector(mut ss) if !ss.is_empty() => match ss.remove(0) {
                    OwnedValue::String(s) => s.to_string(),
                    other => panic!("expected the error as an EDN string; got {other:?}"),
                },
                other => panic!("expected a vector of error strings; got {other:?}"),
            },
            other => panic!("expected a tagged LociDiedError; got {other:?}"),
        },
        other => panic!("expected a vector at the top; got {other:?}"),
    };
    let parsed = wat_edn::parse_owned(&inner_text).expect("inner error must be EDN");
    match parsed {
        OwnedValue::Tagged(tag, body) => {
            assert_eq!(
                tag,
                Tag::ns("wat.runtime", "FixpointRoundCapExceeded"),
                "the refusal must be the TYPED cap error — before the cap this was an allocator \
                 abort with no tag at all"
            );
            match *body {
                OwnedValue::Map(m) => m,
                other => panic!("expected a map body; got {other:?}"),
            }
        }
        other => panic!("expected a tagged error; got {other:?}"),
    }
}

fn field_i64(fields: &[(OwnedValue, OwnedValue)], name: &str) -> i64 {
    let v = fields
        .iter()
        .find(|(k, _)| *k == OwnedValue::Keyword(Keyword::new(name)))
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("the error must carry :{name}; got {fields:?}"));
    match v {
        OwnedValue::Integer(n) => *n,
        other => panic!(":{name} must be an integer; got {other:?}"),
    }
}

#[test]
fn a_non_terminating_rule_set_is_refused_and_names_the_cap() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_fixpoint_round_cap.wat");
    assert!(
        !ok,
        "a rule set that never converges must NOT exit successfully\n{stdout}{stderr}"
    );
    let e = fixpoint_error(&stderr);
    assert_eq!(field_i64(&e, "cap"), 10_000, "the error must name the cap it hit");
    assert_eq!(
        field_i64(&e, "still-deriving"),
        1,
        "and carry the evidence the fixpoint was still GROWING at the cap — that is what \
         separates a runaway rule from a merely deep one"
    );
}

/// The cap is a per-program config value, not a hard constant —
/// `(:wat::config::set-max-fire-rounds! n)`, carried on `Config` and therefore inherited by
/// spawned sub-programs exactly like `dim-count`.
///
/// Why tunable at all: a round count CANNOT distinguish deep from divergent. Transitive closure
/// over a 50_000-node path is legitimate Datalog deriving one level per round, while the cap must
/// stay low enough to fire before the allocator does. Those pressures have no single correct
/// value. Raising it is a claim that your rule set terminates and is merely deep; it is never the
/// fix for one that diverges.
#[test]
fn the_cap_is_a_per_program_config_value() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_fixpoint_round_cap_tuned.wat");
    assert!(!ok, "the runaway must still be refused at the tuned cap\n{stdout}{stderr}");
    let e = fixpoint_error(&stderr);
    assert_eq!(
        field_i64(&e, "cap"),
        25,
        "the engine must honour `(:wat::config::set-max-fire-rounds! 25)` — a 10000 here means the \
         setter parsed but never reached the fire loop, which is the silent half of a config knob"
    );
}

#[test]
fn a_deep_but_terminating_rule_set_still_completes() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_fixpoint_round_cap_deep.wat");
    assert!(
        ok,
        "500 rounds is DEPTH, not divergence — ten times the grid's deepest axis, and it must \
         still run. A cap that fails here is capping a legitimate workload shape.\n{stdout}{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "\"501\"",
        "the deep cascade must derive N(0..500) = 501 facts"
    );
}

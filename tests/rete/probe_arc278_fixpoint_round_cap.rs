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


/// Read a printed `FireOutcome` arm: its name, then its i64 fields, one per line.
///
/// ⛔ WHY THE FIXTURES PRINT INSTEAD OF RAISING. Before the fire-outcome wall a ceiling breach was
/// a raise, and these gates read structured EDN off stderr (`rete_error`). Totality removed the
/// raise: the breach is now an arm the fixture MATCHES, so there is no error to parse and the
/// program exits 0. The arm's fields are the assertion, so the fixture prints them and this reads
/// them back — the same exactness on a value instead of a corpse.
///
/// `println` renders a String with quotes and an i64 bare, which is why the name is compared with
/// them included and the numbers parse cleanly.
fn arm(stdout: &str) -> (String, Vec<i64>) {
    let mut lines = stdout.lines().map(str::trim).filter(|l| !l.is_empty());
    let name = lines
        .next()
        .unwrap_or_else(|| panic!("the fixture must print the arm name first; got {stdout:?}"))
        .trim_matches('"')
        .to_string();
    let fields = lines
        .map(|l| {
            l.parse::<i64>()
                .unwrap_or_else(|e| panic!("arm field must be an i64; got {l:?} ({e})"))
        })
        .collect();
    (name, fields)
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
fn rete_error(stderr: &str, variant: &str) -> Vec<(OwnedValue, OwnedValue)> {
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
                Tag::ns("wat.runtime", variant),
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

fn field_str(fields: &[(OwnedValue, OwnedValue)], name: &str) -> String {
    let v = fields
        .iter()
        .find(|(k, _)| *k == OwnedValue::Keyword(Keyword::new(name)))
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("the error must carry :{name}; got {fields:?}"));
    match v {
        OwnedValue::String(s) => s.to_string(),
        other => panic!(":{name} must be a String; got {other:?}"),
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

/// The runaway is refused by the TERMINATION VERIFIER at `compile-all` — before a fact is
/// inserted, before a round runs. It never reaches the round cap, and the fixture's `"firing..."`
/// never prints: that is the observable difference between "refused at load" and "gave up at run".
///
/// (The old `the_cap_is_a_per_program_config_value` row lived here and is gone deliberately — its
/// fixture was a runaway, which the verifier now catches at compile, so it could no longer reach
/// the cap it was testing. The boundary pair below proves the config knob instead, on a workload
/// the verifier ACCEPTS.)
#[test]
fn a_non_terminating_rule_set_is_refused_at_compile_by_the_verifier() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_fixpoint_round_cap.wat");
    assert!(
        !ok,
        "a rule set that cannot be proven to terminate must NOT compile\n{stdout}{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "",
        "refusal is at COMPILE — nothing in the program body should have run. Output here means \
         the rule set was armed and fired before being caught, which is the cap's job, not the \
         verifier's.\n{stderr}"
    );
    let e = rete_error(&stderr, "RuleSetMayNotTerminate");
    assert_eq!(field_str(&e, "rule"), "cap::grow");
    assert_eq!(field_str(&e, "fact-type"), "cap::N");
}

/// THE FN-HEADED EXPLOIT — the hole 4.2 shipped with, demonstrated and then closed.
///
/// The verifier inspects the `:then` ITEM, and `(:fm::bump ?n)`'s arguments are bare bound
/// variables — the minting is one level down, in the rete fn's body. Before this closed, the
/// fixture compiled clean and ran to the round cap.
///
/// It also pins the two things that made the exploit hard to write, both of which produced FALSE
/// NEGATIVES first: the fn must be declared `:wat::rete::core::defn` (a plain `:wat::core::defn`
/// is refused as a `:then` head for an unrelated reason), and the arithmetic must use the total
/// fallback spelling. Three earlier attempts got one of those wrong, failed for the wrong reason,
/// and briefly convinced me the hole was already guarded by other fences.
///
/// And the analysis had to know that `(:fm::N :k <expr>)` is kwargs SUGAR that macro-expands to
/// `:wat::core::kwargs-construct` — a head starting with `:wat::`, which a "constructors are
/// non-`:wat::` heads" heuristic skips. That one line silently disarmed the whole check.
#[test]
fn a_mint_hidden_inside_a_rete_fn_body_is_refused() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_termination_fn_head.wat");
    assert!(
        !ok,
        "a rete fn whose BODY constructs from a computed value, inside a derivation cycle, mints a \
         novel fact every round — it must not compile\n{stdout}{stderr}"
    );
    let e = rete_error(&stderr, "RuleSetMayNotTerminate");
    assert_eq!(field_str(&e, "rule"), "fm::grow");
    // `fm::N`, NOT `fm::bump`. This row was the whole reason the diagnostic could lie for as long
    // as it did: the gate held the wrong value in its hand and only ever looked at `rule`. The
    // message goes on to say this name "feeds back into this rule's own `:when`" — and the `:when`
    // reads `:fm::N`, so naming the FUNCTION sent the reader hunting for a fact type that does not
    // exist. Found by driving, 2026-08-28; `computed` was built from `fact_type_head` (the raw
    // head) while `produced` beside it resolved through `sym`. One resolver now feeds both.
    assert_eq!(field_str(&e, "fact-type"), "fm::N");
}

/// ★ THE PER-SESSION MEMORY CEILING, AT THE **INSERT** DOOR — staging, with no fire at all.
///
/// ⛔ THIS IS THE HOLE THE BUILDER FOUND. The ceiling was checked only inside the fixpoint, so a
/// session could grow without bound before `fire-rules` was ever called — measured 2026-08-29:
/// **2_500_000 facts staged, no fire, peak RSS 4.0 GB against a 1 GiB contract, no diagnostic.**
/// The ruling: *"the session is the boundary — it may not consume more than the configured amount
/// of memory… insert affects memory just as much."* One contract, two doors.
///
/// The fixture NEVER calls `fire-rules`, which is what makes this a proof about `insert` alone.
#[test]
fn a_session_that_outgrows_its_ceiling_while_staging_is_refused_at_the_insert_door() {
    let (ok, _out, err) = run("tests/rete/probe_arc278_session_memory_ceiling_insert.wat");
    assert!(
        !ok,
        "staging past `max-session-bytes` must be refused even though nothing fires\n{err}"
    );
    let e = rete_error(&err, "SessionMemoryCeilingExceededOnInsert");
    assert_eq!(
        field_i64(&e, "limit"),
        4096,
        "the ceiling reported must be the CONFIGURED one — a hardcoded default here would mean \
         the wat directive is decorative"
    );
    assert!(
        field_i64(&e, "used") > 4096,
        "the reported usage must exceed the limit it tripped; got {}",
        field_i64(&e, "used")
    );
    // ⛔ THE DOOR IS PART OF THE ASSERTION, not decoration. `SessionMemoryCeilingExceeded` (no
    // suffix) is the FIRE door's variant and reports ROUNDS COMPLETED — a number with no meaning
    // where no rounds run. If this gate accepted either tag it would pass just as happily with
    // the insert check deleted, because the old fixture's fire would have refused instead.
    // `staged` is a field the FIRE variant does not have at all, so pinning it proves the door
    // STRUCTURALLY rather than by matching prose. 1, because the breach is detected on the very
    // first `insert`: the fold's own `(range 0 200000)` is already megabytes against a 4096-byte
    // ceiling. That pairing — a large `used` beside `staged: 1` — is the diagnostic working as
    // designed: it says the memory is NOT the facts, which is the honest reading of a counter that
    // measures the thread rather than walking the session.
    assert_eq!(
        field_i64(&e, "staged"),
        1,
        "the insert door must report how far STAGING had got, and this fixture breaches on its \
         first `insert` — a different value means the ceiling is no longer checked per insert"
    );

    // NON-VACUITY: the same insert-only shape under the DEFAULT ceiling completes. Without this,
    // a ceiling of zero — or a check that refuses unconditionally — satisfies everything above.
    let (ok_ok, out_ok, err_ok) = run("tests/rete/probe_arc278_session_memory_ceiling_fire_default.wat");
    assert!(
        ok_ok,
        "the insert door must not refuse a workload that fits — 400 staged facts at the 1 GiB \
         default is nowhere near the ceiling\n{out_ok}{err_ok}"
    );
}

/// ★ THE PER-SESSION MEMORY CEILING, AT THE **FIRE** DOOR — the axis the round cap cannot see.
///
/// The round cap counts ROUNDS. A fanout divergence multiplies WITHIN a round, so it reaches the
/// allocator while `rounds_run` is in single digits — measured 2026-08-29 as `memory allocation of
/// 56 bytes failed` at 6.2s, with no wat error and no rule named. `DEFAULT_MAX_FIRE_ROUNDS`'s own
/// doc had claimed a runaway fires "far short of the memory wall"; that is true of a LINEAR
/// runaway and false of a branching one.
///
/// ⛔ THE FIXTURE'S WORKLOAD CHANGED WHEN `insert` GAINED THE SAME CEILING, and that is the lesson
/// worth keeping: the old one seeded 500 facts at a 4096-byte ceiling, so once staging was
/// enforced the FIRST INSERT refused and this gate silently began proving the other door. It still
/// went green. A control can lose its power without ever failing. The workload is now one the
/// insert door cannot catch — **400 staged, 40_000 derived** — so a green here is evidence about
/// the fixpoint's check and nothing else.
///
/// ⚠ THE CEILING IS BISECTED, NOT PICKED: 1/4/16 MiB refuse, 64/256 MiB complete. 16 MiB sits
/// inside the refusing band with staging orders of magnitude below it.
#[test]
fn a_session_that_outgrows_its_ceiling_while_deriving_is_refused_at_the_fire_door() {
    let (ok, out, err) = run("tests/rete/probe_arc278_session_memory_ceiling.wat");
    // ⛔ THE PROGRAM SURVIVES, AND THAT IS THE POINT OF THE WALL. Before totality this asserted
    // `!ok` — the breach killed the process and the gate read a raise off stderr. `fire-rules` now
    // answers a matchable `(FireOutcome :- [Session])`, so a ceiling breach is a VALUE the fixture
    // handles and the program exits 0. A gate still demanding a corpse would be asserting the
    // absence of the feature.
    assert!(ok, "a ceiling breach is a VALUE now — the program must not die\n{out}{err}");
    let a = arm(&out);
    assert_eq!(
        a.0, "ARM MemoryCeilingExceeded",
        "a fanout that outgrows `max-session-bytes` must take the MEMORY arm, not the round-cap \
         one — they are different mechanisms and this workload multiplies inside one round\n{out}"
    );
    assert_eq!(
        a.1[0], 16_777_216,
        "the ceiling reported must be the CONFIGURED one — a hardcoded default here would mean \
         the wat directive is decorative"
    );
    assert!(
        a.1[1] > 16_777_216,
        "the reported usage must exceed the limit it tripped; got {}",
        a.1[1]
    );
    // `rounds` is rounds COMPLETED, so 0 means "tripped during the first round" — the most
    // informative value it can take. A cross-product fans out inside one round; a non-zero value
    // would mean the check no longer runs before the counter.
    assert_eq!(a.1[2], 0, "a cross product fans out inside the FIRST round\n{out}");

    // NON-VACUITY, doing more work than usual: the byte-for-byte identical rule set and fact
    // population at the DEFAULT ceiling must take the OTHER arm and derive all 40_000. Without it
    // a ceiling of zero, or a check placed before any work, would satisfy every row above.
    let (ok_ok, out_ok, err_ok) = run("tests/rete/probe_arc278_session_memory_ceiling_fire_default.wat");
    assert!(ok_ok, "the same workload at the default ceiling must complete\n{out_ok}{err_ok}");
    assert_eq!(
        out_ok.trim(),
        "40000",
        "the non-vacuity twin must take the `Fired` arm and DERIVE its 40_000 facts — a run that \
         completes without doing the work proves nothing about the ceiling's headroom"
    );

    // The 500-round DEEP closure must also stay under the default — depth is the round cap's axis,
    // and the ceiling must not start refusing legitimate depth.
    let (ok_deep, out_deep, err_deep) = run("tests/rete/probe_arc278_fixpoint_round_cap_deep.wat");
    assert!(
        ok_deep,
        "a 500-round range-restricted closure is legitimate and must not trip the memory \
         ceiling at its default\n{out_deep}{err_deep}"
    );
}

/// ★ A FINITE-TYPED COMPUTED HEAD IS ADMITTED, AND CONVERGES — the eBPF `[u32; 16]` bound,
/// computed instead of declared.
///
/// `(:ft::F :flag (not ?b))` has a fact population of TWO. It was refused for the life of the
/// check, because the cyclicity test measures RANGE RESTRICTION (a syntactic property — the head's
/// value came from the body) and finiteness is a TYPE property. A domain of two was refused exactly
/// as an unbounded `i64` counter is.
///
/// ⛔ THE SECOND HALF IS THE LOAD-BEARING ONE, and it is why this row is not just `assert ok`.
/// Admitting on the wrong axis is far worse than refusing: the `i64` shapes are where the danger
/// was MEASURED — unguarded `i64` reaches an allocator abort in 6.2s with no wat diagnostic,
/// because the round cap counts ROUNDS and a fanout diverges within one. So this row pins that the
/// admission did NOT widen that axis. If a future relaxation lets an `i64` computed head through,
/// this fails here rather than in someone's process.
#[test]
fn a_finite_typed_computed_head_is_admitted_and_the_i64_axis_is_not() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_termination_finite_domain.wat");
    assert!(
        ok,
        "a computed head over a 2-inhabitant type cannot mint unboundedly — it must compile\n{stdout}{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "2",
        "and it must CONVERGE at the population, not merely compile — 2 is {{F(true), F(false)}}, \
         the product of the cardinalities. A number other than 2 means the admission is resting on \
         something other than the bound it claims.\n{stderr}"
    );

    // ⛔ THE CAP IS NOT DECORATIVE. 20 computed `bool` fields is 2^20 = 1_048_576 facts — provably
    // FINITE and, at the measured ~600 B/fact, ~630 MB. "Provably finite" is not "safe to admit",
    // and this row is where those two claims part company. Without it the constant could be set to
    // anything, or deleted, and every other row here would still pass.
    let (ok_big, _o, e_big) = run("tests/rete/probe_arc278_termination_finite_domain_too_large.wat");
    assert!(
        !ok_big,
        "a population past MAX_PROVABLE_FACT_POPULATION must be refused as too large to prove, \
         not admitted because the arithmetic happens to be finite\n{e_big}"
    );
    let _ = rete_error(&e_big, "RuleSetMayNotTerminate");

    // THE AXIS THAT MUST NOT MOVE. Each of these has a computed `i64` head; each is refused today
    // and must stay refused. `_guarded` terminates (at 501) and is refused anyway — that is the
    // KNOWN narrowing, item 8, and it is deliberately still here: this admission reads the TYPE,
    // never the `where` fence.
    for fixture in [
        "tests/rete/probe_arc278_fixpoint_round_cap.wat",
        "tests/rete/probe_arc278_termination_fn_head.wat",
        "tests/rete/probe_arc278_termination_guarded_counter.wat",
    ] {
        let (ok, _out, err) = run(fixture);
        assert!(!ok, "{fixture} must still be refused");
        let e = rete_error(&err, "RuleSetMayNotTerminate");
        let _ = e;
    }
}

/// THE GUARDED COUNTER — refused, and it terminates. Both halves are the assertion.
///
/// `N(k+1) :- N(k), (where (< ?k 500))` halts at k=500 and is refused anyway, because the
/// cyclicity test reads the derivation graph and never the fence. Reported 2026-08-28 by
/// claude-compute; weighed against this tree and confirmed by driving. The refusal is CORRECT by
/// the verifier's own claim — what was missing was a home for the class that could go red.
///
/// Before this, the only record lived in `probe_arc278_fixpoint_round_cap_deep.wat`'s header,
/// describing a fixture that had been REWRITTEN around the refusal. Prose in a file nobody greps.
///
/// If the verifier ever learns to read the fence, this test fails — which is the notification that
/// the narrowing closed, not a regression.
#[test]
fn a_bounded_counter_is_refused_too_and_the_message_does_not_claim_divergence() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_termination_guarded_counter.wat");
    assert!(
        !ok,
        "the structural cyclicity test does not read the `where` fence, so a guarded counter is \
         refused like any other computed head in a cycle\n{stdout}{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "",
        "refusal is at COMPILE — nothing in the program body should have run.\n{stderr}"
    );
    let e = rete_error(&stderr, "RuleSetMayNotTerminate");
    assert_eq!(field_str(&e, "rule"), "gc::count-up");
    assert_eq!(field_str(&e, "fact-type"), "gc::N");

    // THE RETRACTED CLAIM. The message used to assert "the fixpoint can never converge" — false of
    // the very program in front of the user, which converges at k=500. R29 `RVINA ERVDIT`: the
    // ruin must teach, and it may not teach something untrue. The verifier computes a derivation
    // graph; it does not compute convergence, and the diagnostic may not claim what the analysis
    // never established. A targeted absence is the honest shape here — the message is long and
    // will keep being reworded; what may never come back is this sentence.
    // rune:lint(loose-assert) — targeted absence of one retracted phrase in a long, evolving
    // diagnostic; an exact `assert_eq!` on the whole message would pin prose that is meant to be
    // improved, and would go red for rewordings that are not the defect.
    assert!(
        !stderr.contains("can never converge"),
        "the diagnostic must not assert non-convergence — it proves the absence of range \
         restriction in a cycle, which is a refusal to certify, not a proof of divergence\n{stderr}"
    );
}

/// THE BOUNDARY, and it is the row that justifies the deep fixture's size.
///
/// "500 is comfortably under 10,000" tests nothing about the cap's EDGE. A `>` where a `>=`
/// belongs silently costs one round of legitimate depth and no comfortable-margin fixture would
/// ever notice. So the same workload runs at its exact round count and one below.
///
/// The round count is 502, MEASURED by bisecting the cap rather than assumed — the extra two over
/// the 500-edge path are the seed rule's round plus the final no-op round that proves convergence.
/// I had assumed 500; being wrong by two is exactly why the number is pinned here instead of
/// asserted in a comment.
#[test]
fn the_cap_fires_at_exactly_its_round_and_not_one_before() {
    let (ok_pass, out_pass, err_pass) = run("tests/rete/probe_arc278_fixpoint_round_cap_boundary_pass.wat");
    assert!(
        ok_pass,
        "at a cap EQUAL to the workload's round count the fire must complete — refusing here would \
         be off-by-one in the strict direction, silently stealing a round of legitimate \
         depth\n{out_pass}{err_pass}"
    );
    assert_eq!(out_pass.trim(), "\"501\"", "and derive the full closure");

    let (ok_fail, out_fail, err_fail) = run("tests/rete/probe_arc278_fixpoint_round_cap_boundary_fail.wat");
    // Totality: the breach is a VALUE, so the program lives and reports. What must NOT happen is
    // the `Fired` arm — that would be the permissive off-by-one, a cap that let the workload run
    // one round past its bound.
    assert!(ok_fail, "a cap breach is a VALUE now — the program must not die\n{out_fail}{err_fail}");
    let a = arm(&out_fail);
    assert_eq!(
        a.0, "ARM RoundCapExceeded",
        "one round SHORT of what the workload needs must take the ROUND-CAP arm — the `Fired` arm \
         here is off-by-one in the permissive direction\n{out_fail}"
    );
    assert_eq!(
        a.1[0], 501,
        "at the boundary the refusal is the CAP's, not the verifier's — this workload is \
         range-restricted, so the verifier accepts it and only the cap can stop it"
    );
}

#[test]
fn a_deep_but_terminating_rule_set_still_completes() {
    let (ok, stdout, stderr) = run("tests/rete/probe_arc278_fixpoint_round_cap_deep.wat");
    assert!(
        ok,
        "502 rounds is DEPTH, not divergence, and it must still run under the default cap. This \
         fixture is also the TERMINATION VERIFIER's false-positive guard: the rule is plainly \
         cyclic (Reach reads Reach) and must still be ACCEPTED, because `?y` is copied out of Edge \
         rather than computed — range-restricted, finite domain, provable.\n{stdout}{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "\"501\"",
        "the deep cascade must derive N(0..500) = 501 facts"
    );
}

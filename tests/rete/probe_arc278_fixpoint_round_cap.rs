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



// ⛔ `rete_error` / `field_str` / `field_i64` ALL LIVED HERE AND ARE ALL GONE (arc 278). They read
// fields off a RAISED EDN error on stderr. Every refusal this file gates — both ceilings and the
// termination verdict — is now a matchable ARM, so the values arrive on stdout through `arm()` /
// `arm_str()` and nothing in this file reads a corpse any more. Their disappearance IS the wall:
// when the last reader of a raise goes, the raise did too.

/// Read a printed `CompileOutcome::MayNotTerminate` arm: the name, then its two String fields.
///
/// The verifier's verdict is no longer a raise (arc 278) — a rule set built from RUNTIME data
/// cannot be judged before the program runs, so `compile-all` answers a matchable
/// `(:wat::rete::CompileOutcome)` and the fixture prints the arm it took. `println` renders a
/// String with quotes, which is stripped here so the gates compare bare names.
fn arm_str(stdout: &str) -> (String, Vec<String>) {
    let mut lines = stdout.lines().map(str::trim).filter(|l| !l.is_empty());
    let name = lines
        .next()
        .unwrap_or_else(|| panic!("the fixture must print the arm name first; got {stdout:?}"))
        .trim_matches('"')
        .to_string();
    let fields = lines.map(|l| l.trim_matches('"').to_string()).collect();
    (name, fields)
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
// `field_i64` lived here and is GONE (arc 278 S2c). It read an i64 field off a raised EDN error;
// both ceilings are now matchable ARMS, so their numbers arrive as `arm()`'s fields and nothing
// reads them off stderr any more. `rete_error`/`field_str` STAY — `RuleSetMayNotTerminate` is a
// COMPILE-time refusal from the termination verifier, not a ceiling, and it correctly still raises.


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
    // ⛔ THE VERDICT IS A VALUE NOW, so the program LIVES and reports. Before arc 278 this
    // asserted `!ok` and read structured EDN off stderr; `compile-all` answers a matchable
    // `(:wat::rete::CompileOutcome)`, so a refusal exits 0 with the arm on stdout. A gate still
    // demanding a corpse would be asserting the absence of the feature.
    assert!(ok, "a refusal is a VALUE now — the program must not die\n{stdout}{stderr}");
    let a = arm_str(&stdout);
    assert_eq!(
        a.0, "ARM MayNotTerminate",
        "a rule set that cannot be proven to terminate must take the refusing arm\n{stdout}"
    );
    assert_eq!(a.1[0], "cap::grow");
    assert_eq!(a.1[1], "cap::N");
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
    // ⛔ THE VERDICT IS A VALUE NOW, so the program LIVES and reports. Before arc 278 this
    // asserted `!ok` and read structured EDN off stderr; `compile-all` answers a matchable
    // `(:wat::rete::CompileOutcome)`, so a refusal exits 0 with the arm on stdout. A gate still
    // demanding a corpse would be asserting the absence of the feature.
    assert!(ok, "a refusal is a VALUE now — the program must not die\n{stdout}{stderr}");
    let a = arm_str(&stdout);
    assert_eq!(
        a.0, "ARM MayNotTerminate",
        "a mint hidden inside a rete fn body must still take the refusing arm\n{stdout}"
    );
    assert_eq!(a.1[0], "fm::grow");
    assert_eq!(a.1[1], "fm::N");
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
    let (ok, out, err) = run("tests/rete/probe_arc278_session_memory_ceiling_insert.wat");
    // Totality (S2c): staging answers `(:wat::rete::InsertOutcome)`, so a breach is a VALUE and
    // the program lives. What must NOT happen is the `Inserted` arm — that would mean 200_000
    // facts staged under a 4096-byte ceiling with nothing noticing, which is the exact hole this
    // gate was built for.
    assert!(ok, "a staging breach is a VALUE now — the program must not die\n{out}{err}");
    let a = arm(&out);
    assert_eq!(
        a.0, "ARM MemoryCeilingExceeded",
        "staging past `max-session-bytes` must take the ceiling arm — the `Inserted` arm here \
         means `insert` is unbounded again\n{out}"
    );
    assert_eq!(
        a.1[0], 4096,
        "the ceiling reported must be the CONFIGURED one — a hardcoded default here would mean \
         the wat directive is decorative"
    );
    assert!(
        a.1[1] > 4096,
        "the reported usage must exceed the limit it tripped; got {}",
        a.1[1]
    );
    // ⛔ `staged` IS THE DOOR, STRUCTURALLY. The FIRE arm has no such field — it reports `rounds` —
    // so pinning this proves which door refused without matching on prose. 1, because the breach
    // is caught on the very first `insert`: the fold's own `(range 0 200000)` is already megabytes
    // against a 4096-byte ceiling. That pairing — a large `used` beside `staged: 1` — is the
    // diagnostic working as designed: it says the memory is NOT the facts, which is the honest
    // reading of a counter that measures the thread rather than walking the session.
    assert_eq!(
        a.1[2], 1,
        "the insert door must report how far STAGING had got, and this fixture breaches on its \
         first `insert` — a different value means the ceiling is no longer checked per insert"
    );

    // NON-VACUITY: a workload that FITS must take the `Inserted` arm and complete. Without this,
    // a ceiling of zero — or a check that refuses unconditionally — satisfies everything above.
    let (ok_ok, out_ok, err_ok) = run("tests/rete/probe_arc278_session_memory_ceiling_fire_default.wat");
    assert!(
        ok_ok,
        "the insert door must not refuse a workload that fits — 400 staged facts at the 1 GiB \
         default is nowhere near the ceiling\n{out_ok}{err_ok}"
    );
    assert_eq!(
        out_ok.trim(),
        "40000",
        "and that workload must actually do its work"
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
    let (ok_big, o_big, e_big) = run("tests/rete/probe_arc278_termination_finite_domain_too_large.wat");
    assert!(ok_big, "the verdict is a VALUE — the program must not die\n{o_big}{e_big}");
    assert_eq!(
        arm_str(&o_big).0,
        "ARM MayNotTerminate",
        "a population past MAX_PROVABLE_FACT_POPULATION must be refused as too large to prove, \
         not admitted because the arithmetic happens to be finite\n{o_big}"
    );

    // THE AXIS THAT MUST NOT MOVE. Each of these has a computed `i64` head; each is refused today
    // and must stay refused. `_guarded` terminates (at 501) and is refused anyway — that is the
    // KNOWN narrowing, item 8, and it is deliberately still here: this admission reads the TYPE,
    // never the `where` fence.
    for fixture in [
        "tests/rete/probe_arc278_fixpoint_round_cap.wat",
        "tests/rete/probe_arc278_termination_fn_head.wat",
        // ⛔ `..._termination_guarded_counter.wat` WAS IN THIS LIST AND IS DELIBERATELY GONE
        // (item 8, 2026-08-29). It IS a computed `i64` head — but it carries a `where` fence
        // pointing against its step, so the MEASURE proof admits it now. Leaving it here would
        // re-assert the narrowing this arc just closed. What remains is the axis that must not
        // move: a computed `i64` head with NO fence, which nothing can bound.
    ] {
        let (ok, out, err) = run(fixture);
        // Refused means the REFUSING ARM, not a dead process — the verdict is a value (arc 278).
        assert!(ok, "{fixture}: the verdict is a VALUE — it must not die\n{out}{err}");
        assert_eq!(
            arm_str(&out).0,
            "ARM MayNotTerminate",
            "{fixture} must still be refused — its head computes an `i64`, and the finite-domain \
             admission reads the TYPE, never a `where` fence\n{out}"
        );
    }
}

/// ★ A FENCE-BOUNDED COUNTER IS ADMITTED — and its one-character twin is not.
///
/// ⛔ THIS TEST ASSERTED THE OPPOSITE, AND THAT IS THE POINT. It was
/// `a_bounded_counter_is_refused_too_…`, gating a KNOWN NARROWING: the cyclicity test is structural
/// and read no `where` fence, so `N(k+1) :- N(k), (where (< ?k 500))` — which halts at 500 — was
/// refused. The old gate said, in its own words: *"if the verifier ever learns to read the fence,
/// this test fails — which is the notification that the narrowing closed, not a regression."*
/// **It fired. This is that notification, collected.**
///
/// Reported from OUTSIDE this tree (the main × grok-rete integration branch,
/// `~/work/NOTE-rete-termination-verifier-refuses-provably-bounded-recursion.md`) — which is why it
/// mattered: someone wrote the first thing anyone writes in recursive Datalog-with-arithmetic and
/// was refused.
///
/// ⚠ WHAT MADE IT ADMISSIBLE WAS NOT A CLEVERER PROOF — IT WAS THE RUNTIME CEILINGS. This class
/// proves TERMINATION but not POPULATION SIZE: the seed is runtime data, so `k` may start at
/// -10^18 and the fixpoint, though finite, is enormous. Before `max-session-bytes` existed that
/// meant an allocator abort with no diagnostic, and "terminates" was not enough to admit on. The
/// ceiling now catches size as a located, matchable value. **The verifier proves termination; the
/// ceilings bound cost. Neither could admit this class alone** — exactly the trade item 8 recorded
/// as its own precondition, and met by this arc's own ceiling work.
#[test]
fn a_fence_bounded_counter_is_admitted_and_its_wrong_way_twin_is_not() {
    // The terminating shape: `k+1` while `k < 500`. Admitted — AND it converges. A rule set that
    // compiles and then hangs would satisfy any weaker assertion here.
    let (ok, out, err) = run("tests/rete/probe_arc278_termination_guarded_counter.wat");
    assert!(ok, "a fence-bounded counter must compile and run\n{out}{err}");
    assert_eq!(
        out.trim(),
        "\"501\"",
        "and CONVERGE at 501 — the seed plus every step up to the bound. Admitting a rule set that \
         then hangs would be worse than refusing it\n{err}"
    );

    // ⛔ THE SOUNDNESS TWINS. One character from the admitted shapes, and they DIVERGE: `k+1` while
    // `k > 500` produces a value that also satisfies `> 500`, forever. Without these rows the
    // analysis could accept ANY fence and every other assertion here would still pass.
    for (fixture, why) in [
        (
            "tests/rete/probe_arc278_termination_fence_bad_up.wat",
            "`k+1` under a LOWER bound steps AWAY from it — the guard is satisfied forever",
        ),
        (
            "tests/rete/probe_arc278_termination_fence_bad_down.wat",
            "`k-1` under an UPPER bound steps away likewise",
        ),
    ] {
        let (ok_t, out_t, err_t) = run(fixture);
        assert!(ok_t, "{fixture}: the verdict is a VALUE — it must not die\n{out_t}{err_t}");
        assert_eq!(
            arm_str(&out_t).0,
            "ARM MayNotTerminate",
            "{fixture} must STAY refused — {why}\n{out_t}"
        );
    }

    // The decreasing mirror is admitted too, so the analysis cannot be written for one direction
    // and quietly refuse the other half of the shape it claims to cover.
    let (ok_d, out_d, err_d) = run("tests/rete/probe_arc278_termination_fence_ok_down.wat");
    assert!(ok_d, "the decreasing mirror must compile\n{out_d}{err_d}");
    assert_eq!(
        out_d.trim(),
        "\"ADMITTED\"",
        "`k-1` while `k > 0` terminates and must be admitted\n{err_d}"
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

/// ⚠ DISCONFIRMING PROBE — vigilia Class A4, RED until the session origin is keyed per session.
///
/// `alloc_counter::SESSION_ORIGIN` is ONE `Cell` per THREAD, set unconditionally by
/// `mark_session_origin` from `arm-session` — which every `compile-all` reaches. A second session
/// therefore REBASES the zero point, and everything the first had already staged stops being
/// charged to it.
///
/// Two wards found this independently on 2026-08-30 (`secare` hunting shared state, `sequi`
/// hunting state threading). The module's own doc states the assumption — *"one session per thread
/// at a time"* — and `arm_lease.rs` holds two live sessions on one thread in a GREEN test, so the
/// assumption was already false when it was written.
///
/// Both arms stage the SAME 16,000 facts into ONE session. The only difference is one unrelated
/// `compile-all` between the two staging rounds.
#[test]
fn a_second_session_on_the_thread_does_not_forgive_the_first_its_ceiling() {
    let (ok, out, err) =
        run("tests/rete/probe_arc278_session_ceiling_second_session.wat");
    assert!(ok, "a staging breach is a VALUE — the program must live\n{out}{err}");

    let lines: Vec<&str> = out.lines().map(str::trim).collect();
    let verdict = |tag: &str| -> String {
        let i = lines
            .iter()
            .position(|l| l.trim_matches('"') == tag)
            .unwrap_or_else(|| panic!("fixture must print `{tag}`\n{out}"));
        lines[i + 1].trim_matches('"').to_string()
    };

    // The control pins the ceiling as live at this workload — without it, a green probe arm
    // would prove nothing, because "nothing breached" is also what an unreachable ceiling looks
    // like. This is the row that makes the probe's claim falsifiable.
    assert_eq!(
        verdict("control"),
        "REFUSED",
        "the ceiling must refuse 16,000 staged facts when nothing intervenes — if this says \
         NO-BREACH the workload no longer crosses 4 MB and the probe below is vacuous\n{out}"
    );
    assert_eq!(
        verdict("probe"),
        "REFUSED",
        "THE CEILING STOPPED ENFORCING. The identical 16,000-fact workload was refused above and \
         admitted here, and the only difference is one unrelated `compile-all` between the two \
         staging rounds. `mark_session_origin` rebased this thread's single `SESSION_ORIGIN`, so \
         everything the session had already staged stopped being charged to it — and once \
         `thread_bytes()` falls below that new origin, `saturating_sub` floors the reading at 0 \
         and the session has no ceiling at all for the rest of its life\n{out}"
    );
    // THE ARM KEYING ALONE CANNOT SEE. `rearm` hands the SAME session back to `arm-session`
    // mid-life, so the second `mark_session_origin` arrives under the FIRST session's own key —
    // and a store that keys its origins but still overwrites them is indistinguishable from the
    // fix at both arms above. Added 2026-08-30 after the strike's prescribed mutation ("make
    // `mark_session_origin` clobber regardless of id") was measured INERT: with distinct keys,
    // `insert` and `or_insert` do the same thing and control/probe both stayed green. Under
    // `or_insert` -> `insert` this row, and only this row, reads NO-BREACH.
    assert_eq!(
        verdict("rearm"),
        "REFUSED",
        "THE CEILING STOPPED ENFORCING ON RE-ARM. The same session was handed back to \
         `arm-session` between the two staging rounds, and its origin was overwritten instead of \
         kept — so everything staged before the re-arm stopped being charged to it. An origin is \
         written ONCE per session and never moved; `mark_session_origin` must not clobber a key \
         it already holds\n{out}"
    );
}

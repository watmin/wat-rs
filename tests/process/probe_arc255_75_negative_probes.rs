//! Arc 255 Stone 255.75 — "every probe runs" (ruling E3): "A probe that must fail is a
//! `.wat.bad` with a driven test naming its error." This file drives the nine probes
//! whose claim IS a refusal/crash.
//!
//! AMEND: all ten (at the time) were first renamed `.wat.bad` in `wat-scripts/probes/`, per the
//! brief's literal reading of E3 — but `tests/lint/every_wat_bad_fixture_actually_fails.rs`
//! (pre-existing, not this stone's) drives `startup_from_file` on every `.wat.bad` and requires
//! it to refuse AT STARTUP; all ten failed only at RUNTIME (division by zero, a dispatch miss, a
//! refused builtin call, …), so they `startup_from_file` CLEAN and that gate correctly
//! red-flagged all ten: "the file is a valid program and the NAME is wrong — git mv it to
//! `.wat`". Per that remedy, each moved to a plain `.wat` under `tests/process/fixtures/` (OUT of
//! `wat-scripts/probes/`, so the new "every probe runs, exit 0" gate does not see it), each with
//! its own header DISPOSITION/AMEND note. One of the ten, `probe-m1-fix-revoke.wat`, was then
//! RETIRED (coordinator correction, 2026-10-01): its claim pins a specific raw substrate outcome
//! (a clean `Closed` EOF) that the M1-TEETH design itself treats as equivalent to `Lost`/`Stopped`
//! (all three fold to `Outcome.Bounced` — `tests/services/probe_arc170_m1_teeth_revoked.wat:112-117`,
//! per `DESIGN-STONE-M1-TEETH-revoke-refusal.md`), exactly as `probe-m1-cf-norevoke.wat` was
//! retired — see the brief's dispositions #11/#12 in the SCORE. Nine remain here. Per probe
//! below: what it claims, why it must fail, and what the assertion pins.
//!
//! `needle_count` mirrors `tests/types/probe_arc255_74_key_must_be_data.rs`'s own
//! `needle_count` shape — `.matches(needle).count()`, never a loose `.contains()` inside
//! an assert (`no_loose_string_assert`'s own remedy, `tests/lint/no_loose_string_assert.rs`).

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Runs the release binary against `rel` (relative to the crate root), stdin `/dev/null`.
/// Returns `(exit_code, stdout, stderr)`.
fn run(rel: &str) -> (Option<i32>, String, String) {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join(rel);
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&path)
        .current_dir(&manifest)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn needle_count(hay: &str, needle: &str) -> usize {
    hay.matches(needle).count()
}

// ─── probe-bracket-cause.wat.bad — the crash message must name the REAL cause ──────────

#[test]
fn bracket_cause_names_the_real_cause() {
    let (code, _stdout, stderr) = run("tests/process/fixtures/probe-bracket-cause.wat");
    assert_ne!(code, Some(0), "a runner that divides by zero must crash: {stderr}");
    // AMEND (floor red, 2026-10-01): the bracket maps 3 workers in parallel; under full-floor
    // concurrent load a DIFFERENT worker (measured: runner 1, not runner 0) can be the one
    // collect-loop observes crashing first — real scheduling non-determinism over WHICH worker
    // reports, not over whether the message names ITS crash. "runner 0" was an incidental detail
    // from one isolated run, never part of the probe's own claim; check for "runner " + "crashed"
    // as two needles (never a specific index) rather than the single literal "runner 0 crashed".
    assert!(
        needle_count(&stderr, "runner ") >= 1 && needle_count(&stderr, " crashed:") >= 1,
        "the collect-loop assertion must name which runner crashed: {stderr}"
    );
    assert!(
        needle_count(&stderr, "DivisionByZero") >= 1,
        "the message must carry the REAL cause (DivisionByZero), not a blind \"runner crashed\": {stderr}"
    );
}

// ─── probe-cap2-peer-pid.wat.bad — peer-pid on a connect-derived unified Peer errors ───

#[test]
fn cap2_peer_pid_on_unified_peer_is_an_honest_error() {
    let (code, stdout, stderr) = run("tests/process/fixtures/probe-cap2-peer-pid.wat");
    assert_ne!(
        code,
        Some(0),
        "peer-pid on a connect'-derived unified Peer must error (DESIGN-STONE-CAP-2-BRACKET-GRANTS.md's \
         grounded finding — peer-pid is for spawn-derived peers only): {stderr}"
    );
    assert_eq!(
        needle_count(&stdout, "process-peer peer-pid:"),
        1,
        "the println right before the gap must still have printed: {stdout}"
    );
    assert!(
        needle_count(&stderr, ":wat::kernel::peer-pid") >= 1,
        "the error must name peer-pid as the offending op: {stderr}"
    );
    assert!(
        needle_count(&stderr, "expected peer") >= 1,
        "the error must name the expected-peer union type: {stderr}"
    );
}

// ─── probe-child-inherits-defns.wat.bad — decisive: the child is a fresh universe ──────

#[test]
fn child_inherits_defns_is_refused() {
    let (code, _stdout, stderr) =
        run("tests/process/fixtures/probe-child-inherits-defns.wat");
    assert_ne!(
        code,
        Some(0),
        "a not-shared process child does NOT inherit the parent's named defns: {stderr}"
    );
    assert!(
        needle_count(&stderr, "UnresolvedReferences") >= 1,
        "the decisive answer is a resolve-time UnresolvedReferences, not a runtime NameError: {stderr}"
    );
    assert!(
        needle_count(&stderr, ":probe::dbl") >= 1,
        "the error must name the exact unresolved reference: {stderr}"
    );
}

// ─── probe-defclause-discriminate.wat.bad — defclause has no open-surface fallback ─────

#[test]
fn defclause_has_no_open_surface_fallback() {
    let (code, _stdout, stderr) =
        run("tests/process/fixtures/probe-defclause-discriminate.wat");
    assert_ne!(
        code,
        Some(0),
        "an unknown concrete class (MongoReason) must NOT be caught by an open-surface \
         [r <- :probe::Reason] clause — dispatch is exact-class only \
         (tests/rete/probe_arc278_open_surface_dispatch.rs already rules this): {stderr}"
    );
    assert!(
        needle_count(&stderr, "NoMatchingClause") >= 1,
        "the runtime dispatcher must raise NoMatchingClause: {stderr}"
    );
    assert!(
        needle_count(&stderr, "MongoReason") >= 1,
        "the error must name the unmatched concrete class: {stderr}"
    );
    assert!(
        needle_count(&stderr, ":probe::describe") >= 1,
        "the error must name the defclause that failed to dispatch: {stderr}"
    );
}

// ─── probe-fnforms-keyword-err.wat.bad — fn-forms refuses an unregistered keyword ──────

#[test]
fn fnforms_keyword_err_is_refused() {
    let (code, stdout, stderr) =
        run("tests/process/fixtures/probe-fnforms-keyword-err.wat");
    assert_ne!(
        code,
        Some(0),
        "fn-forms given a keyword naming no registered fn must be refused: {stderr}"
    );
    assert!(
        needle_count(&stderr, ":wat::kernel::fn-forms") >= 1,
        "the error must name fn-forms as the offending op: {stderr}"
    );
    assert!(
        needle_count(&stderr, ":no::such::fn") >= 1,
        "the error must name the bogus keyword: {stderr}"
    );
    assert_eq!(
        needle_count(&stdout, "should not reach here"),
        0,
        "the println past the fn-forms call must never run: {stdout}"
    );
}

// ─── probe-m1-addr-roundtrip.wat.bad — a capability tag never reconstructs from parsed data ──

#[test]
fn m1_addr_roundtrip_is_refused_by_the_capability_wall() {
    let (code, stdout, stderr) =
        run("tests/process/fixtures/probe-m1-addr-roundtrip.wat");
    assert_ne!(
        code,
        Some(0),
        "edn/read must refuse to reconstruct an Address' capability tag from parsed data: {stderr}"
    );
    assert_eq!(
        needle_count(&stdout, "wire:"),
        1,
        "the wire-form println before the gap must still have printed: {stdout}"
    );
    assert!(
        needle_count(&stderr, "unsupported substrate tag") >= 1,
        "the error must name the unsupported-tag wall: {stderr}"
    );
    assert!(
        needle_count(&stderr, "Address") >= 1,
        "the error must name the Address tag: {stderr}"
    );
    assert!(
        needle_count(&stderr, "capability tags reconstruct only off the trusted peer wire") >= 1,
        "the error must carry the security rationale, not just a bare refusal: {stderr}"
    );
}

// ─── probe-s1-impure-gate.wat.bad — fn-forms never leaks an impure Process capture ─────

#[test]
fn s1_impure_gate_never_leaks() {
    let (code, stdout, stderr) = run("tests/process/fixtures/probe-s1-impure-gate.wat");
    assert_ne!(
        code,
        Some(0),
        "fn-forms must refuse to reify a closure that captures a live Process handle: {stderr}"
    );
    assert!(
        needle_count(&stderr, "closure-extract") >= 1,
        "the error must come from closure-extract: {stderr}"
    );
    assert!(
        needle_count(&stderr, "not implemented") >= 1,
        "the error must name the slice-1 not-yet-implemented wall: {stderr}"
    );
    assert!(
        needle_count(&stderr, ":wat::kernel::Process") >= 1,
        "the error must name the captured value's kind: {stderr}"
    );
    assert_eq!(
        needle_count(&stdout, "LEAK"),
        0,
        "the leak println must never run — that is this probe's whole claim: {stdout}"
    );
}

// ─── probe-generic-shipped.wat.bad — a declared-generic shipped runner can't call a ──────
// ─── monomorphic helper and claim the abstract result type ──────────────────────────────

#[test]
fn generic_shipped_runner_cannot_claim_the_abstract_type() {
    let (code, _stdout, stderr) = run("tests/process/fixtures/probe-generic-shipped.wat");
    assert_ne!(
        code,
        Some(0),
        "a pool-runner declared generic over [A B] cannot call the monomorphic __work \
         (reified from a concrete i64->i64 fn) and claim its argument/result has the abstract \
         type B: {stderr}"
    );
    // AMEND (coordinator correction, 2026-10-01): the fixture's RecvOutcome bare-bind +
    // missing SendOutcome.Stopped (255.73's defect class) is now repaired in place, same as the
    // other five probes — the pool-runner's body matches on `recv` and the `send` match carries
    // the Stopped arm. That repair also UN-MASKS a 3rd generic-class error
    // (`:bracket::__work`'s own parameter check) that the bare-bind's own type error had been
    // hiding: 5 originally measured → 3 remain, all load-bearing, none from the repaired class.
    assert!(
        needle_count(&stderr, "3 type-check errors") >= 1,
        "the child's own startup type-check must report exactly the 3 load-bearing \
         generic-vs-monomorphic errors, with the RecvOutcome-bare-bind/Stopped-arm class \
         repaired: {stderr}"
    );
    assert!(
        needle_count(&stderr, ":wat::core::Tuple: parameter #2 expects :B; got :wat::core::i64") >= 1,
        "the Tuple constructor must refuse the abstract-B slot (out's 2nd element): {stderr}"
    );
    assert!(
        needle_count(&stderr, ":bracket::__work: parameter #1 expects :wat::core::i64; got :B") >= 1,
        "the monomorphic __work call must refuse the abstract-B argument: {stderr}"
    );
    assert!(
        needle_count(
            &stderr,
            ":wat::kernel::send: parameter payload expects :(wat::core::i64,A); got :(wat::core::i64,B)"
        ) >= 1,
        "send must refuse out's (i64,B) payload against self's declared (i64,A) send type: {stderr}"
    );
}

// ─── probe-s3b-crux-fnforms-closure.wat.bad — fn-forms can't reify a closure that captures a fn ──

#[test]
fn s3b_crux_fnforms_closure_cannot_reify_a_captured_fn() {
    let (code, _stdout, stderr) =
        run("tests/process/fixtures/probe-s3b-crux-fnforms-closure.wat");
    assert_ne!(
        code,
        Some(0),
        "fn-forms of a closure that CAPTURES a fn value must be refused \
         (NOTE-S3b-blockers-and-resolution.md names this probe RED BY DESIGN): {stderr}"
    );
    assert!(
        needle_count(&stderr, "closure-extract") >= 1,
        "the error must come from closure-extract: {stderr}"
    );
    assert!(
        needle_count(&stderr, "not implemented") >= 1,
        "the error must name the slice-1 not-yet-implemented wall: {stderr}"
    );
}

//! `a-defclause-outranks-a-defn` — **a consumer may not re-point a library's own names.**
//!
//! DESIGN: `docs/excursus/2026/08/001-sns-sqs/a-defclause-outranks-a-defn/DESIGN.md`
//! ROOT:   `docs/excursus/2026/08/001-sns-sqs/can-a-user-def-change-a-stdlib-verdict/FINDING.md`
//!
//! ## What was open
//!
//! A `defclause`'s clause table is consulted at `check.rs`'s call-site dispatch BEFORE the
//! registered scheme, and at runtime `runtime_def_values` outranks `sym.functions` the same
//! way. Nothing checked whether the name was already declared. So a CONSUMER that declares
//! clauses for `:mylib::greet` after loading `mylib` re-pointed that name for the LIBRARY's
//! own bodies — and the library author had no defence, because only `:wat::` / `:rust::` /
//! `:$bound::` are reserved.
//!
//! ⛔ **Two doors, and the second one was silent.** Over a plain `defn` the consumer's clause
//! produced a type error — badly located (inside the library) but loud. Over a name a TYPE
//! generates (the per-field accessor, the primed positional constructor) it produced NOTHING:
//! `register_defclause`'s `Stub` phase takes the name at freeze step 5, companion codegen at
//! step 6.8a finds it occupied and declines to mint, and the library's own accessor call then
//! runs consumer code with no type error at all. Measured before the wall: `7` became `1004`
//! and the consumer's `println` fired, **exit 0**. That is behaviour substitution, not a type
//! error, and it is what the accessor/ctor cases below hold shut.
//!
//! ## Why REFUSE and not EXTEND
//!
//! A form-aware census (`wat-scripts/grep/defclause-over-defn.wat`) over all 1873 tracked
//! `.wat` files found 71 `defclause` declarations and **zero** that name something also
//! declared in the same program — same-file or cross-file. Nothing in the corpus extends a
//! `defn` with a clause, so extending buys a capability no caller has while leaving the
//! silent-substitution door open (a clause that MATCHES still changes which body runs).
//! ARMED AT ZERO OFFENDERS, the same pattern as `UnreachableClause`.
//!
//! ## The diagnostic is half the fix
//!
//! Before: `NoMatchingClauseAtCallSite`, located INSIDE the library — blaming a file the
//! person who must fix it never wrote. After: one error located at the CONSUMER's own form,
//! naming the declaration it would displace. `the_refusal_is_located_in_the_consumers_file`
//! is what holds that, and it is the test that would go red if someone "simplified" the wall
//! back into the call-site check.
//!
//! ## Fixtures
//!
//! Co-located, never inlined. The two libraries are fed to an `InMemoryLoader` under the
//! names the consumers load (`lib.wat`, `lib-record.wat`, `clause-lib.wat`) — this is the
//! LIBRARY-AND-CONSUMER shape the stone is about, which a single-file fixture cannot express.
//! `.wat.bad` marks the three consumers that must be refused.

use std::sync::Arc;
use wat::freeze::{startup_from_source, FrozenWorld, StartupError};
use wat::load::loader::InMemoryLoader;

const DIR: &str = "tests/function/";
const STEM: &str = "probe_defclause_over_defn_refused";

fn fixture(suffix: &str) -> String {
    let path = format!("{DIR}{STEM}{suffix}");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {path:?} must exist (run from crate root): {e}"))
}

/// Freeze a consumer fixture with the two libraries reachable through `load-file!`.
fn freeze_consumer(consumer_suffix: &str) -> Result<FrozenWorld, StartupError> {
    let mut loader = InMemoryLoader::new();
    loader.add_source("lib.wat", fixture("_lib.wat"));
    loader.add_source("lib-record.wat", fixture("_lib_record.wat"));
    loader.add_source("clause-lib.wat", fixture("_clause_lib.wat"));
    startup_from_source(
        &fixture(consumer_suffix),
        Some("consumer.wat"),
        Arc::new(loader),
    )
}

fn call_i64(world: &FrozenWorld, fn_name: &str) -> i64 {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no entry fn {fn_name:?} in the frozen world"))
        .clone();
    match wat::runtime::apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(wat::Value::i64(n)) => n,
        other => panic!("{fn_name}: expected an i64, got {other:?}"),
    }
}

fn refusal_text(consumer_suffix: &str, why: &str) -> String {
    let err = freeze_consumer(consumer_suffix).expect_err(why);
    format!("{err:?}")
}

// ── door 1: a clause over a library `defn` ───────────────────────────────────────────

#[test]
fn consumer_clause_over_a_library_defn_is_refused() {
    let rendered = refusal_text(
        "_neg_defn.wat.bad",
        "a consumer must not declare clauses for a name the library already declared",
    );
    for needle in [
        "ClauseOverExistingDeclaration",
        ":mylib::greet",
        "already declared as a function",
        "OUTRANKS",
    ] {
        assert!(
            rendered.contains(needle),
            "the refusal must name {needle:?} — got: {rendered}"
        );
    }
}

/// ★ THE DIAGNOSTIC TEST. The blame must sit on the form the author can edit.
///
/// Before the wall this program failed with `NoMatchingClauseAtCallSite` whose location was
/// the library — a file the consumer never wrote and cannot change. All three halves are
/// asserted: the consumer's file IS the location, the library is still CITED as the displaced
/// declaration, and the old call-site complaint is GONE.
#[test]
fn the_refusal_is_located_in_the_consumers_file_not_the_library() {
    let rendered = refusal_text("_neg_defn.wat.bad", "must be refused");
    for needle in [
        // the located span — the consumer's own form, which is the thing to edit
        ":file \"consumer.wat\"",
        // the displaced declaration, cited (as :prior-decl-loc) but not blamed
        ":prior-decl-loc",
        ":file \"lib.wat\"",
    ] {
        assert!(
            rendered.contains(needle),
            "the refusal must carry {needle:?} — got: {rendered}"
        );
    }
    // rune:lint(loose-assert) — a targeted ABSENCE over the whole rendered error tree; the
    // claim is "this error kind no longer appears anywhere in it", which has no exact form.
    assert!(
        !rendered.contains("NoMatchingClauseAtCallSite"),
        "the library's own call site must no longer be the thing that complains — got: {rendered}"
    );
}

// ── door 2: a clause over a name a TYPE generates — the silent one ───────────────────

/// Non-vacuity for the pair below: with the consumer's clause REMOVED, this answers 7.
#[test]
fn accessor_baseline_answers_seven() {
    let world = freeze_consumer("_pos_accessor.wat")
        .expect("the library and a consumer that adds nothing must freeze");
    assert_eq!(call_i64(&world, ":user::probe"), 7);
}

/// ⛔ THE SILENT ONE. Before the wall this froze clean and answered **1004** — the library's
/// own accessor call ran the consumer's body, with no type error anywhere.
#[test]
fn consumer_clause_over_a_library_record_accessor_is_refused() {
    let rendered = refusal_text(
        "_neg_accessor.wat.bad",
        "a clause on a record's generated accessor must be refused, not silently obeyed",
    );
    for needle in [
        "ClauseOverGeneratedCompanion",
        ":mylib::Point/x",
        "per-field accessor",
    ] {
        assert!(
            rendered.contains(needle),
            "the refusal must name {needle:?} — got: {rendered}"
        );
    }
}

/// The same door, the other companion: the primed positional constructor.
#[test]
fn consumer_clause_over_a_positional_constructor_is_refused() {
    let rendered = refusal_text(
        "_neg_ctor.wat.bad",
        "a clause on a record's positional constructor must be refused",
    );
    for needle in ["ClauseOverGeneratedCompanion", "positional constructor"] {
        assert!(
            rendered.contains(needle),
            "the refusal must name {needle:?} — got: {rendered}"
        );
    }
}

// ── the lines the wall must NOT cross ────────────────────────────────────────────────

/// A `defclause` on a name nobody else declared is the ordinary case — 71 of them in this
/// corpus. If this goes red the wall has started refusing `defclause` itself.
#[test]
fn a_defclause_on_a_fresh_name_still_registers_and_dispatches() {
    let world = freeze_consumer("_pos_fresh.wat")
        .expect("a defclause on an undeclared name must still register");
    assert_eq!(call_i64(&world, ":user::probe"), 42);
}

/// ★★ THE REGRESSION THE WALL WOULD MOST EASILY CAUSE. `register_defclause`'s `Stub` phase
/// registers a `Function` under the defclause's OWN name, so by check time EVERY defclause
/// name is in `sym.functions` — a bare `has_function` test would refuse all 71 of them and
/// break this: loading one file twice re-registers the same form, and that is a LIVE, green
/// path (measured before the wall existed: `42`, exit 0).
///
/// The discriminator is the stub's body span, which IS the defclause form's span. If this
/// goes red, someone replaced that identity test with a shape heuristic.
#[test]
fn loading_one_file_twice_is_still_green() {
    let world = freeze_consumer("_pos_double_load.wat").expect(
        "loading one file twice must stay green — the wall must not read the second \
         registration of the SAME form as a displacement",
    );
    assert_eq!(call_i64(&world, ":user::probe"), 42);
}

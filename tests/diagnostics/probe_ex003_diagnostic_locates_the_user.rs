//! Probe (excursus 003) — A DIAGNOSTIC'S `:file` MUST NAME THE USER'S PROGRAM, NEVER OURS.
//!
//! `the-little-wat` flagged this twice, a week apart, both by accident:
//!
//! - **F-006** — an unknown-type refusal carries `:file "src/check.rs"`. *"The user is told what
//!   is wrong but not where."* **CURED** by excursus 003 stone B; the test below now pins the
//!   cure, and `f114` and `variant_singleton` beside it pin the rest of that stone.
//! - **F-091** — a lex error carries `:file "crates/wat-reader/src/parser.rs"` and gives no line
//!   or column of the user's file at all, only a byte offset. It has a measured cost: finding
//!   which of 532 files raised `lex error at byte 258` meant grepping all 532 OUTSIDE wat.
//!   **STILL OPEN** — a different mechanism (the reader, not the checker), its own stone.
//!
//! Both reproduced at HEAD when this probe was banked. ⭐ F-006's span had already MOVED — their
//! ledger records `src/check.rs:15141`, it was `15285` the day it was driven. The line drifted;
//! the defect did not. A stale line number in a ledger reads exactly like a fixed defect, which
//! is why these are driven and not quoted.
//!
//! ## What stone B changed, and the boundary it does NOT cross
//!
//! `TypeEnv` now retains the declaration span arc 138 already threaded into
//! `register_validated` (`decl_spans`), and the two post-registration walks read it instead of
//! `rust_caller_span!()`. A `defn` has no `TypeEnv` row, so that arm uses the function body's own
//! `WatAST` span.
//!
//! ⚠ **The contract is the DECLARATION, not the offending token.** `TypeExpr` carries no span,
//! so the annotation's own location never reaches the registry; these tests therefore assert the
//! control's shape — *no `.rs` file* — and deliberately do not pin a column.
//!
//! ⚠ **A diagnostic naming a BUILTIN can still name a `.rs` file**, because a builtin really is
//! declared in Rust. These four fixtures all declare their own types in wat, so none of them can
//! reach that arm.
//!
//! ## The root: the sentinel is a CONVENTION, NOT A SHAPE
//!
//! `src/check/error.rs` states the doctrine: *"`crate::rust_caller_span!()` is the explicit
//! sentinel for the rare site with no recoverable source location; `Display`/`diagnostic()`
//! elide unknown spans."* Measured against that sentence, on this tree:
//!
//! - **"rare" is 597** `rust_caller_span!()` call sites under `src/`.
//! - **"elide" is not what the user sees** — the EDN face emits the sentinel verbatim, which is
//!   how F-006 and F-091 reach a reader at all.
//!
//! `src/span.rs` shows why nothing catches it: the macro expands to an ordinary `Span` carrying
//! Rust's `file!()`/`line!()`. There is no sentinel TYPE. A sentinel and a real user location are
//! the same shape, so only string-sniffing can tell them apart — and that is rung one of the
//! ladder, which is where this defect lives.
//!
//! ## The invariant this probe pins, and why it is not "not under `src/`"
//!
//! ⛔ **`:file` must never name a `.rs` file.** The tree's own existing mask,
//! `blank_rust_source_lines` (`src/lib.rs:271-275`), tests `starts_with("src/") &&
//! ends_with(".rs")` — so `crates/wat-reader/src/parser.rs` **is not masked**, and F-091's span
//! would survive into a captured golden today. Keying on the `.rs` suffix alone covers both
//! without enumerating roots, and closes that hole by construction.
//!
//! ## ⛔ ONE of these tests STILL PINS A KNOWN DEFECT and goes RED when cured. That is the point.
//!
//! `f091_*` asserts the defect. The other four assert the CURED shape, and the first of them is a
//! NEGATIVE CONTROL that must pass in every world: without it the detector could pass by flagging
//! every diagnostic, and would stay green after a "cure" that broke user spans.
//! `[[a-resolver-whose-halves-overlap-proves-nothing]]`
//!
//! ⭐ The three cured tests are mutation-proved together: forcing `decl_span()` /
//! `body_span()` back to `crate::rust_caller_span!()` in `src/check.rs` turns all three RED and
//! leaves the control green. A gate that has never failed is not a gate.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// ⛔ `ext` is not decoration — a fixture that is DELIBERATELY unparseable must be named
/// `.wat.bad`, or `tests/lint/every_tracked_wat_parses.rs` reds the floor over it.
///
/// ⚠ **That gate enumerates with `git ls-files`, so it CANNOT SEE AN UNSTAGED FILE.** This probe
/// landed red at `f2e328ff6` after its author ran that very gate and watched it pass — the
/// fixture was untracked at the time, became tracked at commit, and the gate only then had a
/// subject. Running a tracked-corpus gate BEFORE `git add` proves nothing.
/// `[[a-file-landing-in-a-gated-tree-needs-that-gate-run]]`
fn fixture(case: &str, ext: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/diagnostics")
        .join(format!("probe_ex003_diagnostic_locates_the_user__{case}.{ext}"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

fn stderr_of(case: &str, ext: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(case, ext))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// Every `:file "…"` value in a diagnostic, at either escaping depth.
///
/// A top-level `:location` renders `:file "src/check.rs"`; one nested inside a `:message` string
/// renders `:file \"…\"`. Both are read here — a reader that saw only one depth would report the
/// nested case as having no location at all.
fn span_files(stderr: &str) -> Vec<String> {
    let mut out = Vec::new();
    for seg in stderr.split(":file ").skip(1) {
        let seg = seg.trim_start().trim_start_matches('\\');
        let seg = match seg.strip_prefix('"') {
            Some(rest) => rest,
            None => continue,
        };
        let end = seg.find(['"', '\\']).unwrap_or(seg.len());
        out.push(seg[..end].to_string());
    }
    out
}

/// The subset of those that name one of OUR Rust files rather than the user's program.
///
/// Keyed on the `.rs` suffix alone — see the module header on why `starts_with("src/")` is the
/// wrong key and what it already misses.
fn our_own_source(stderr: &str) -> Vec<String> {
    let mut v: Vec<String> = span_files(stderr)
        .into_iter()
        .filter(|f| std::path::Path::new(f).extension().is_some_and(|e| e == "rs"))
        .collect();
    v.sort();
    v.dedup();
    v
}

/// NEGATIVE CONTROL — a refusal that already locates correctly, and must in every world.
///
/// NON-VACUITY: the extractor is proven to find something here before either defect test is
/// allowed to mean anything; an empty parse would make both of those pass by finding nothing.
#[test]
fn control_a_runtime_refusal_locates_the_users_own_file() {
    let err = stderr_of("control_user_span", "wat");
    let files = span_files(&err);
    assert!(
        !files.is_empty(),
        "the span extractor found NO :file at all — it has stopped reading the diagnostic \
         format, and the two defect tests below would pass by measuring nothing.\n{err}"
    );
    assert_eq!(
        our_own_source(&err),
        Vec::<String>::new(),
        "a diagnostic that used to name the user's file now names wat-rs's own source — \
         this is the control, so a cure has broken user spans.\nfiles seen: {files:?}"
    );
}

/// Assert one fixture's diagnostic locates the USER, in the control's shape.
///
/// NON-VACUITY is not optional here: `our_own_source` returning empty is ALSO what a diagnostic
/// with no `:file` at all looks like, and what an unchanged `:message` format looks like, so the
/// extractor is made to prove it found a location before its verdict is allowed to mean anything.
/// `[[a-suite-notices-a-deleted-instrument-only-through-nonzero-assertions]]`
fn assert_locates_the_user(case: &str, ext: &str, what: &str) {
    let err = stderr_of(case, ext);
    let files = span_files(&err);
    assert!(
        !files.is_empty(),
        "{what}: the span extractor found NO :file at all — either the refusal stopped firing \
         or the diagnostic format moved, and the verdict below would measure nothing.\n{err}"
    );
    assert_eq!(
        our_own_source(&err),
        Vec::<String>::new(),
        "{what}: the refusal names wat-rs's own source instead of the user's declaration.\n\
         files seen: {files:?}\n{err}"
    );
}

/// F-006 — the unknown-type refusal, raised by `validate_named_type_annotations`'s
/// `functions_iter()` arm (this program declares a `defn`, not a type), names the user's file.
#[test]
fn f006_an_unknown_type_refusal_names_the_users_own_file() {
    assert_locates_the_user(
        "f006_unknown_type",
        "wat",
        "F-006 (UnknownNamedType, function arm)",
    );
}

/// F-114 — the containment-rule refusal, raised by `validate_aggregate_containment`'s
/// `env.iter()` walk, names the user's `defrecord`.
///
/// ⚠ This pins F-114's LOCATION only. Its message still calls a function type an "impure
/// (struct) type" and reasons entirely about structs, and it still carries no `:remedies` key.
/// Those are message CONTENT and are a separate stone by design — bundling them here would hide
/// which change fixed what.
#[test]
fn f114_an_impure_field_refusal_names_the_users_own_file() {
    assert_locates_the_user(
        "f114_impure_field",
        "wat",
        "F-114 (ImpureFieldInPureAggregate, type arm)",
    );
}

/// STOP-1, found and driven while curing the two above: a VARIANT SINGLETON had no declaration
/// span, because `register_variant_type` is a sibling door that bypasses `register_validated`.
///
/// This one is not merely wrong, it was UNSTABLE — the parent enum and its singleton sit side by
/// side in `env.iter()`, so HashMap order decided which was refused first and the same program
/// reported the user's file on some runs and `src/check.rs` on others (8 runs split 4/4). A
/// singleton now inherits its enum's span, which is the declaration a reader would be sent to.
#[test]
fn variant_singleton_a_refusal_names_the_enums_declaration() {
    assert_locates_the_user(
        "variant_singleton",
        "wat",
        "variant singleton (UnknownNamedType via register_variant_type)",
    );
}

/// F-091 — the lex error names wat-rs's reader, and gives no user line or column.
#[test]
fn f091_a_lex_error_still_names_wat_rs_own_reader() {
    let err = stderr_of("f091_lex_error", "wat.bad");
    let ours = our_own_source(&err);
    assert!(
        !ours.is_empty(),
        "F-091 is CURED — the lex error no longer names a .rs file. Go close the finding.\n{err}"
    );
}

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
//!   **CURED** by excursus 003 stone O; `f091` below now pins the cure.
//!
//! A third mechanism joined the same census (154 of 166 errors already located in the user's
//! file; the loader was 1 of the misses), found and cured directly as stone P rather than first
//! banked as a the-little-wat finding:
//!
//! - **Stone P** — `(:wat::load-file! "no-such-file.wat")` carried `:file "src/load/loader.rs"`:
//!   `From<LoadFetchError> for LoadError` stamped `crate::rust_caller_span!()` on every fetch
//!   failure even though `process_single_load` already held the `load-file!` form's own span.
//!   **CURED** by excursus 003 stone P; `stone_p_*` below pin the cure.
//!
//! Both F-006 and F-091 reproduced at HEAD when this probe was banked. ⭐ F-006's span had already MOVED — their
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
//! ## What stone O changed, for F-091
//!
//! `crates/wat-reader/src/lexer.rs`'s `LexError` stays a bare byte `position` at all ~43 of its
//! construction sites — untouched. `lex`/`lex_with_comments` now attach a real `Span` exactly
//! ONCE, at their own return boundary, via a new `LocatedLexError { span, error }` built from the
//! `file`/`compute_line_starts`/`line_col` already in scope there (the same machinery every token
//! span goes through). `From<LocatedLexError> for ParseError` (`parser.rs`) reads that span
//! instead of calling `crate::rust_caller_span!()`. `span.end` is `None` (point-span): `position`
//! is a verified char boundary, but guessing a byte-width end risks slicing a multi-byte char
//! mid-boundary (see `crates/wat-reader/tests/reader_totality.rs`), so this cure names the file,
//! line and column and does not also claim to bound a range.
//!
//! ## What stone P changed, and the ONE site it could not cure the same way
//!
//! `impl From<LoadFetchError> for LoadError` (`src/load/loader.rs`) is DELETED outright, not
//! patched — with no blanket `From`, no future `?` on a `LoadFetchError` can silently reach for
//! the sentinel again. `fetch_source` / `fetch_payload` now take the caller's `form_span` and
//! `map_err` explicitly; `verify_pre_parse` / `verify_post_parse` gained a `form_span` parameter
//! for the same reason; the `LoadErrorKind::Parse` site in `process_single_load` swapped its
//! `rust_caller_span!()` for the same `form_span`. All four sites the brief named had a real
//! `load-file!` form span in reach.
//!
//! A FIFTH call site did not: `crates/wat-macros/src/lib.rs`'s `wat::main! { loader: … }`
//! expansion built a `LoadError` from `ScopedLoader::new`'s failure via `LoadError::from(e)` —
//! the deleted blanket `From` — and this fires BEFORE any `.wat` form is ever read, when the
//! `loader:` ROOT PATH itself (a Rust-side macro argument, not wat source) fails to canonicalize.
//! No `form_span` exists here BY CONSTRUCTION, not by an oversight: there is no form yet. The
//! fix is an EXPLICIT `LoadError::new(::wat::rust_caller_span!(), …)` at that one site — not a
//! silent `?`, and driven, not assumed: because the tokens are built by `quote!` inside a
//! `#[proc_macro]`, `rust_caller_span!()`'s `file!()`/`line!()` resolve to the CONSUMER crate's
//! own `wat::main! { … }` invocation (its call-site span), not to `wat-macros/src/lib.rs` — e.g.
//! a broken `loader: "does-not-exist"` in `examples/with-loader/src/main.rs` reports `:location
//! #wat.core/Span {:file "examples/with-loader/src/main.rs" :line 1 :col 1}`, the embedding
//! program's own macro call, never wat-rs's own source. That is the one recoverable location a
//! misconfigured loader root has.
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
//! ## ⛔ Every defect this probe was banked (or later extended) to pin is now CURED — every test
//! below asserts the cured shape. That is not a reason to delete it: it is the standing
//! regression fixture.
//!
//! All seven assert the CURED shape (no `.rs` `:file`), and the first of them is a NEGATIVE
//! CONTROL that must pass in every world: without it the detector could pass by flagging every
//! diagnostic, and would stay green after a "cure" that broke user spans.
//! `[[a-resolver-whose-halves-overlap-proves-nothing]]`
//!
//! ⭐ The three stone-B tests are mutation-proved together: forcing `decl_span()` / `body_span()`
//! back to `crate::rust_caller_span!()` in `src/check.rs` turns all three RED and leaves the
//! control green. `f091` (stone O) has its own mutation: forcing `From<LocatedLexError> for
//! ParseError` (`crates/wat-reader/src/parser.rs`) back to `crate::rust_caller_span!()` turns
//! ONLY `f091` RED and leaves the control and the three stone-B tests green. The two `stone_p_*`
//! tests have their own mutation: restoring `impl From<LoadFetchError> for LoadError` (with
//! `rust_caller_span!()`, `src/load/loader.rs`) and having `fetch_source` lean on `?` again turns
//! ONLY the two `stone_p_*` tests RED, leaving the control, the three stone-B tests, and `f091`
//! green. A gate that has never failed is not a gate.

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

/// Strip the `:frames [...]` vector's text before scanning for `.rs` files that must NOT
/// appear.
///
/// Excursus 003 D3 (envelope step 2) deliberately gives every diagnostic a `:frames` list
/// that CAN — is meant to — name Rust source: one `:Rust` frame (`#[track_caller]` at the
/// raising site) always, plus any `:Wat` frame whose call crossed from Rust (e.g. the entry
/// call `apply_function` makes from `src/freeze.rs`). That is D3's whole point — "like
/// clojure has java in its traces" — and this module's OWN control fixture reproduced it the
/// moment `:frames` was wired into `RuntimeError::to_edn()`: `our_own_source` on the
/// control's stderr went from `[]` to `["src/collection/eval.rs", "src/freeze.rs"]` with no
/// change to this file. This control's contract was always about `:location` (and any other
/// non-`:frames` field) naming the user's program — `:frames` is a different field with a
/// different, ALREADY-RULED contract, so it is excluded here rather than the control's bar
/// being lowered.
fn strip_frames_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(idx) = rest.find(":frames [") {
        out.push_str(&rest[..idx]);
        let after = &rest[idx + ":frames [".len()..];
        let mut depth: i32 = 1;
        let mut end = after.len();
        for (i, c) in after.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// The subset of those that name one of OUR Rust files rather than the user's program.
///
/// Keyed on the `.rs` suffix alone — see the module header on why `starts_with("src/")` is the
/// wrong key and what it already misses. `:frames` is stripped first (see
/// `strip_frames_field`) — that field's Rust locations are BY DESIGN, not this control's
/// subject.
fn our_own_source(stderr: &str) -> Vec<String> {
    let scoped = strip_frames_field(stderr);
    let mut v: Vec<String> = span_files(&scoped)
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

/// F-091 — the lex error, raised for a name ending in `<` (`AngleTypeHeadInName`), now names
/// the user's own `.wat` file, at the offending byte's real line and column.
#[test]
fn f091_a_lex_error_names_the_users_own_file() {
    assert_locates_the_user("f091_lex_error", "wat.bad", "F-091 (lex error, AngleTypeHeadInName)");
}

/// Stone P — `(:wat::load-file! "does-not-exist-stone-p.wat")`. `From<LoadFetchError> for
/// LoadError` used to name `src/load/loader.rs` (`rust_caller_span!()`) even though
/// `process_single_load` already held the `load-file!` form's own span.
#[test]
fn stone_p_a_missing_load_names_the_load_call() {
    assert_locates_the_user(
        "load_missing",
        "wat",
        "stone P (LoadErrorKind::Fetch, missing file)",
    );
}

/// Stone P — a lex error in a LOADED file. `assert_locates_the_user` alone cannot tell "two
/// different non-`.rs` files" from "the same file twice", so this checks the split directly: the
/// OUTER `:location` must be the `load-file!` call (this fixture); the nested `:cause`'s own
/// `:location` (stone O's `LocatedLexError`) must be the LOADED file's real line/col — outer is
/// where the user asked to load, inner is where it broke.
#[test]
fn stone_p_a_lex_error_in_a_loaded_file_locates_outer_and_cause() {
    let err = stderr_of("load_lex_error", "wat");
    let files = span_files(&err);
    assert_eq!(
        files,
        vec![
            "tests/diagnostics/probe_ex003_diagnostic_locates_the_user__load_lex_error.wat"
                .to_string(),
            "tests/diagnostics/probe_ex003_diagnostic_locates_the_user__load_lex_error_target.wat.bad"
                .to_string(),
        ],
        "stone P (LoadErrorKind::Parse, lex error in loaded file): expected exactly the outer \
         load-file! call's file followed by the loaded file's own lex-error location.\n{err}"
    );
}

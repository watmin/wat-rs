//! DISCONFIRMING PROBE — vigilia Class D1: a misspelled enum variant in a rete constraint
//! compiles, fires, and matches nothing, with no diagnostic.
//!
//! `validate/typing.rs`'s `keyword_constant_segment` types a bare keyword constant by PREFIX only
//! and never checks the variant exists, so `:evt::G::Hii` types as "enum" and the rete checker
//! passes it. The runtime resolves through `sym.unit_variant` — an EXACT lookup — gets `None`, and
//! falls back to a plain keyword. `enum::=` then compares Enum vs keyword: always false.
//!
//! ⛔ CORE REFUSES THE IDENTICAL EXPRESSION at check time. `matcher::enum_variant_ctor` already
//! exists as the one resolution, documented "ONE COPY … hand-written at THREE independent sites".

use std::path::Path;
use std::process::{Command, Stdio};

/// ⚠ RUNS FROM THE MANIFEST DIR WITH A RELATIVE PATH, deliberately. It used to `join` the
/// manifest dir and pass an ABSOLUTE path — fine while every arm only read the exit code, but a
/// refusal's `Span` carries `:file` verbatim, so the absolute form makes the diagnostic
/// MACHINE-DEPENDENT and no `.edn` golden over it could ever be checked in.
fn run(rel: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(bin)
        .arg(rel)
        .current_dir(manifest)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {rel} in {}: {e}", manifest.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The control. Without it, a green probe arm below is indistinguishable from a rule that never
/// fired for some unrelated reason — "matched nothing" is also what a broken fixture looks like.
#[test]
fn a_real_enum_variant_in_a_rete_constraint_matches() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo.wat");
    assert!(ok, "the control fixture must run\n{out}{err}");
    assert_eq!(
        out.trim(),
        "1",
        "`:evt::G::Hi` exists and exactly one seeded Req carries it — if this is not 1 the \
         fixture drifted and the probe below proves nothing\n{out}"
    );
}

/// ⚠ EXPECTED RED until Class D1 lands.
#[test]
fn a_misspelled_enum_variant_in_a_rete_constraint_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo_bad.wat");
    assert!(
        !ok,
        "SILENT WRONG ANSWER: a rule constraining on `:evt::G::Hii` — a variant the enum does not \
         declare — compiled, fired, and printed {:?} with exit 0 and no diagnostic. Core REFUSES \
         the identical expression at check time (`parameter #2 expects :wat::core::keyword; got \
         :evt::G`), so the two engines disagree about the same input and rete ships the wrong \
         answer. A typo became a constraint that compiles, fires, and matches nothing.\n{out}{err}",
        out.trim()
    );
}

/// ⚠ ARM 2 — the arm the obvious fix does NOT close. `enum_variant_ctor` resolves Unit **and**
/// Tagged, so routing through it alone still types the bare `:tg::P::Hi` (arity 1) as an `enum`
/// while the runtime's `sym.unit_variant` is UNIT-ONLY and yields a plain keyword. The typing must
/// additionally require **arity == 0**. EXPECTED RED until that lands.
#[test]
fn a_bare_tagged_enum_variant_in_a_rete_constraint_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo_tagged.wat");
    assert!(
        !ok,
        "SILENT WRONG ANSWER: a rule constraining on the BARE tagged variant `:tg::P::Hi` — which \
         has no bare value form at all, `(:tg::P::Hi 7)` is the only way to write one — compiled, \
         fired, and printed {:?} with exit 0 and no diagnostic. Core REFUSES the identical \
         expression at check time (`parameter #2 expects [:wat::core::i64 :-> :tg::P]`), so the \
         two engines disagree about the same input and rete ships the wrong answer.\n{out}{err}",
        out.trim()
    );
}

// ─── strike-variant-diagnostic — the refusal must NAME THE MISTAKE ───────────────────────────
//
// D1 (above) made the typo REFUSE. It refused through the keyword-constant route, where an
// operand naming no declared field comes out as a located `UnknownField`, so the message read
// *"`:evt::Req` has no field `:evt::G::Hii`; available fields: [k, grade]"* — the author sent
// hunting for a FIELD when they mistyped a VARIANT, and handed the record's field names as the
// remedy. Core does not name it either (`TypeMismatch`, `:remedies []`), so agreement with core
// was never the target. **A confidently wrong remedy costs more than none.**

/// ★ THE STRIKE. The refusal names the enum, the variant AS WRITTEN, and the variants that exist.
///
/// The golden is exact, which is what makes the ABSENCE half enforceable: a message that listed
/// the real variants AND the field names — the wrong remedy wearing a right one — passes any
/// presence-only check and fails this. The two targeted absences below say that intent out loud,
/// because an exact golden alone does not record WHICH difference was the finding.
#[test]
fn the_misspelled_variant_refusal_names_the_enum_and_its_real_variants() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo_bad.wat");
    assert!(!ok, "the misspelling must still refuse — D1's ground\n{out}{err}");
    wat::assert_edn_eq!(
        err.trim().to_string(),
        include_str!("probe_arc278_enum_variant_typo_bad__refusal.edn"),
        "the refusal must be `#wat.rete/UnknownEnumVariant` carrying `:enum-path \"evt::G\"`, \
         `:variant \"Hii\"` and `:available-variants [\"Hi\" \"Lo\"]` — captured whole, so a \
         reordered field, an appended remedy, or a fallback to `UnknownField` all fail here"
    );
    // rune:lint(loose-assert) — a targeted ABSENCE over a large output. The finding IS this
    // substring: the old refusal offered `available fields: [k, grade]` for a variant typo, and a
    // probe that only asserts the variants appear would pass on a message carrying both lists.
    assert!(
        !err.contains("available fields"),
        "the remedy must be the VARIANTS, not the fields — the confidently-wrong remedy is back\n{err}"
    );
    // rune:lint(loose-assert) — a targeted ABSENCE over a large output: `grade` is the record
    // field the old message offered, and no field name belongs in a variant diagnostic.
    assert!(
        !err.contains("grade"),
        "a record field name leaked into the variant diagnostic\n{err}"
    );
}

/// ROW 4, driven rather than assumed: the TAGGED path does **NOT** reach the new arm.
///
/// `:tg::P::Hi` (arity 1) RESOLVES through `enum_variant_ctor` — the variant exists, it simply has
/// no bare value form — so `classify_keyword_constant` returns `Keyword` and D1's `UnknownField`
/// route is unchanged. That is deliberate: *"`:tg::P` has no variant `Hi`; available variants:
/// [Hi]"* would be a false statement inside a diagnostic built to stop false statements in
/// diagnostics. **Its message is still the wrong remedy** — it offers `available fields:
/// [k, grade]` — which `strike-variant-diagnostic/DESIGN.md` affirmatively cuts from this strike
/// and which this arm PINS rather than blesses, so the day someone takes that cut, this test is
/// the one that goes red and names what changed.
///
/// ⚠ THE GOLDEN'S SPAN MOVED, 2026-09-01, and the message did not. `strike-field-span` made
/// `UnknownField` carry the FIELD's span: `:col 31 :end 76` (46 characters — the whole
/// `(:wat::rete::core::enum::= :grade :tg::P::Hi)`) became `:col 65 :end 75`, which is
/// `:tg::P::Hi` at its own column, length 10. That is this fixture's own pre-value in that
/// strike's scorecard, driven here. The remedy this arm pins is still wrong and still uncut —
/// only the caret is now on the token the author actually mistyped.
#[test]
fn the_bare_tagged_variant_keeps_the_unknown_field_route() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo_tagged.wat");
    assert!(!ok, "the bare tagged variant must still refuse — D1's arm 2\n{out}{err}");
    wat::assert_edn_eq!(
        err.trim().to_string(),
        include_str!("probe_arc278_enum_variant_typo_tagged__refusal.edn"),
        "the tagged arm's landing, captured whole and NOT endorsed: `#wat.rete/UnknownField` with \
         `:available-fields [\"k\" \"grade\"]`. Naming it is row 4 of the scorecard; fixing it is \
         a separate strike"
    );
}

/// ROW 5 — the failure that would pass every probe above. An arm that fired on any `::` name, or
/// on any keyword at all, would refuse CORRECT programs while the strike's own probes went green.
///
/// Three routes to `KeywordConstant::Keyword`, one rule each, all three firing: no `::`; a `::`
/// whose prefix names nothing; a `::` whose prefix names a registered AGGREGATE. The count is 3
/// because each rule targets a distinct row — identical derived facts would dedup and make two
/// firing rules indistinguishable from one.
#[test]
fn legitimate_keyword_constants_are_still_keywords() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo_keyword.wat");
    assert!(ok, "no legitimate keyword constant may be refused by the new arm\n{out}{err}");
    assert_eq!(
        out.trim(),
        "3",
        "all three keyword-constant routes must still compile AND fire — a count below 3 means \
         the new refusal, or a widening of it, ate a legitimate constant\n{out}{err}"
    );
}

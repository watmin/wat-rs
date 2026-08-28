// DISCONFIRMING PROBE for `RETE-OPEN-WORK.md` § 4.1 — the `RETE_OPS` reachability ledger.
//
// **THE CLAIM UNDER TEST:** a rule exercising one `RETE_OPS` row at one CALL-SITE KIND can be
// synthesized from the row's own metadata, driven through the real load path, and observed to
// either FIRE or be REFUSED — so a 74-row x N-kind ledger can be generated rather than
// hand-written as ~300 fixtures.
//
// **What would REFUTE it:** the synthesized source failing for reasons unrelated to the row under
// test (a grammar the template gets wrong, a type the record cannot declare), so that "refused"
// stops meaning "a user cannot reach this row here" and starts meaning "my template is broken".
// That is the whole risk, and it is why this file is a CALIBRATION before it is a ledger.
//
// ─── Why the ledger's unit is (row x call-site kind) and not the row ────────────────────────────
//
// Rows are already gated for purity, totality, arity and type (`vocabulary.rs`'s
// `every_rete_row_is_total` and siblings). NOTHING gates "can a user actually get here". The
// motivating case is `:wat::rete::core::keyword::=`, and it defeats both obvious ledger designs:
// a GREP ledger calls it dead (it appears in two scratch-pad files); a COMPILES-SOMEWHERE ledger
// calls it fine (those files compile and fire). Both are wrong, because reachability is not a
// property of the ROW — that op is reachable inside a `(:wat::rete::where …)` fence and NOT as an
// inline alpha constraint, and the difference is a real defect a user cannot infer.
// See `docs/arc/2026/04/109-kill-std/NOTE-keyword-is-two-disjoint-type-names-…md`.
//
// ─── The calibration principle, and it is FM 28/29 made into a rule ────────────────────────────
//
// ⛔ **AN INSTRUMENT THAT HAS NOT REPRODUCED A KNOWN ANSWER IS NOT AN INSTRUMENT.** Every cell
// this file reports is a claim about what a user can write. A template that silently mis-renders
// one position would report REFUSED for every row there and read exactly like a discovery — a
// whole column of false findings that look like the jackpot. So the probe below pins FOUR cells
// whose answers are already known from the disk, two of each verdict, and they are the control:
// if any of them moves, nothing else this file says counts.
//
// ─── Why this drives synthesized source instead of `.wat` fixtures ─────────────────────────────
//
// ⚠ `freeze::startup_from_file`'s doc states the repo's fixture doctrine: test wat lives in `.wat`
// files, *"NEVER inlined as a Rust string"*, because a real fixture can be `cargo wat`-run,
// fix-wat-migrated and lint-checked. That doctrine is about a STORED fixture drifting from the
// language. It is deliberately not followed here, and the reason is that following it would be
// WEAKER, not merely inconvenient:
//
//   · These cells are generated at test time, so there is nothing to rot. If the rete grammar
//     moves, the generator goes RED — where ~300 stored fixtures would be silently swept by a
//     `fix-wat` codemod into still-passing files that no longer test the original shape.
//   · A checked-in corpus is three parts that must stay in step (generator, files, a byte-identity
//     regeneration gate) and refused cells cannot share a file with firing ones. One moving part
//     beats three; that is the Simple question, and the corpus design fails it.
//   · `startup_from_forms`' own doc names *"dynamically generated tests"* as a legitimate caller,
//     and `startup_from_file` slurps into `startup_from_source` — so this drives the IDENTICAL
//     chokepoint a user's file does. The vantage is the caller's; only the INVENTORY is internal.
//
//   The UX half of the doctrine is real and is paid for explicitly: every failure prints the
//   complete source it drove, so a failing cell is one paste away from a scratch `.wat`.
//
// Run: cargo nextest run --release -E 'test(reachability)'

use crate::freeze::startup_from_source;
use crate::runtime::{apply_function, Value};
use std::sync::Arc;

/// WHERE a rete op is written. The ledger's second axis — the thing the keyword case proves is
/// not derivable from the row.
///
/// Only the two positions with a KNOWN divergence are modelled here. `:then` and the user
/// accumulator fold are the other two the vocabulary's own module doc names ("every rete verb a
/// `where` / `:then` / user accum fold may call") and are deliberately not guessed at yet — an
/// un-calibrated position would add a column of findings nobody can trust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallSite {
    /// Inside a fact pattern, beside its bindings: `(:R (?k <- :k) (OP :v 10))`.
    InlineConstraint,
    /// Inside a fence clause of its own: `(:wat::rete::where (OP ?v 10))`.
    WhereFence,
}

impl CallSite {
    fn label(self) -> &'static str {
        match self {
            CallSite::InlineConstraint => "inline-constraint",
            CallSite::WhereFence => "where-fence",
        }
    }
}

/// What the ledger records for one cell. A cell with no verdict is the defect the ledger exists
/// to make impossible; there is deliberately no `Unknown`.
///
/// ⛔ **`Refused` AND `TemplateDefect` ARE THE SAME OBSERVATION AND OPPOSITE FINDINGS**, and
/// keeping them apart is the single most important thing this type does. The load path said no
/// either way. The question is WHO it said no to:
///
///   · the refusal NAMES the op under test  -> a real answer about the rete surface -> `Refused`
///   · it names something else entirely     -> my synthesized program is malformed  -> `TemplateDefect`
///
/// **This distinction was bought, not designed.** The first version of this file had only
/// `Refused`, and the calibration's own keyword cell came back carrying TWO errors: the genuine
/// `ConstraintTypeNotComparable`, and an `UnknownField` for `:alpha` — because in INLINE operand
/// position a bare keyword is read as a FIELD REFERENCE, never a keyword value
/// (`matcher.rs`'s `ast_literal_value`, and arc 109's NOTE names it as its "third wrinkle"). So
/// the cell was half-refused for a reason belonging to the template. With one variant that cell
/// still reads REFUSED and the ledger looks right. Scale that to 74 rows and a template that is
/// subtly wrong in one position reports a whole COLUMN of refusals that read exactly like a
/// discovery — the most expensive possible failure of this instrument, since its findings are
/// meant to be believed.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// A rule was written at this position, compiled, fired, and selected the row it should.
    Fires,
    /// The load path refused it AND the diagnostic named the op under test. A real answer: a
    /// user cannot reach this row here. The message is carried because the REASON is the finding —
    /// refused for "no comparator for this type" is a different defect from "that is not a rete
    /// head".
    Refused(String),
    /// The cell's own program is at fault. This is NOT a finding about rete, it must be loud
    /// rather than counted, and it is never an expected outcome.
    ///
    /// The KIND is carried structurally rather than left to be grepped out of the detail string.
    /// That is not tidiness: the first draft asserted `msg.contains("nope")` to tell one defect
    /// cause from another, which `no_loose_string_assert` correctly flagged — a `contains` over a
    /// 2KB diagnostic passes on reordered fields and appended garbage, and it made the test's real
    /// question ("WHICH way did the cell break?") answerable only by string-matching another
    /// subsystem's wording. Naming the causes makes the question exact and leaves the wording free
    /// to improve without reddening anything.
    TemplateDefect(DefectKind, String),
}

/// The ways a cell's own program can be at fault — every one of them a reason to fix the
/// generator, never to record a verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DefectKind {
    /// Something refused it, but the diagnostic never named the op under test — so the refusal
    /// is about the template, not about whether a user can reach this op here.
    Unattributed,
    /// The program loaded but carried no `:probe::run` to drive.
    NoEntryFn,
    /// It compiled and FIRED, but selected a row count the constraint does not admit. The op was
    /// reached and then did nothing — which is not reachability, it is a template that forgot to
    /// discriminate, and counting it as `Fires` would be the ledger's worst false positive.
    DidNotDiscriminate,
    /// The entry returned something that is not a count at all.
    NonCount,
}

/// The knobs one cell needs. Hand-fed for the calibration; derived from `ReteOp::params`/`ret`
/// once the mechanism is proven.
struct Cell {
    /// The rete-surface FQDN under test.
    op: &'static str,
    /// wat type of the discriminating field.
    field_ty: &'static str,
    /// Literal for the fact that SHOULD survive the constraint.
    hit: &'static str,
    /// Literal for the fact that should NOT.
    miss: &'static str,
    /// Right-hand operand as written INLINE (where a bare keyword is a field reference, never a
    /// keyword value — `matcher.rs`'s `ast_literal_value`).
    inline_rhs: &'static str,
    /// Right-hand operand as written inside a fence (ordinary expression grammar; no field-ref
    /// reading, which is half of why the two positions diverge).
    where_rhs: &'static str,
}

/// Build the complete program for one cell.
///
/// Both arms differ ONLY in the condition vector — same records, same facts, same query, same
/// entry point. That is deliberate: it means a difference in outcome between two cells of the
/// same row is attributable to the POSITION and to nothing else.
fn synth(cell: &Cell, site: CallSite) -> String {
    let condition = match site {
        CallSite::InlineConstraint => {
            format!("(:probe::In (?k <- :k) ({} :v {}))", cell.op, cell.inline_rhs)
        }
        CallSite::WhereFence => format!(
            "(:probe::In (?k <- :k) (?v <- :v))\n   (:wat::rete::where ({} ?v {}))",
            cell.op, cell.where_rhs
        ),
    };
    format!(
        r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- {field_ty}])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [{condition}]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q
  :params []
  :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::rete::insert session (:probe::In :k "hit"  :v {hit}))
     session (:wat::rete::insert session (:probe::In :k "miss" :v {miss}))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#,
        field_ty = cell.field_ty,
        condition = condition,
        hit = cell.hit,
        miss = cell.miss,
    )
}

/// Drive one synthesized program and return its verdict.
///
/// A refusal can land at EITHER boundary and both are the same answer to "can a user reach this
/// op here": rule validation runs at freeze (`startup_from_source` -> `Err`), while the compile
/// fence raises at rule-compile time (a panic out of `apply_function`). The caller must not care
/// which — only that the form did not become a live rule. Both fold into `Refused`.
fn drive(src: &str, op: &str) -> Verdict {
    let world = match startup_from_source(src, None, Arc::new(crate::load::InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => return attribute(format!("{e:?}"), op),
    };
    let Some(func) = world.symbols().get(":probe::run").cloned() else {
        return Verdict::TemplateDefect(DefectKind::NoEntryFn, "no `:probe::run`".to_string());
    };
    let sym = world.symbols();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, crate::rust_caller_span!())
    }));
    match outcome {
        Ok(Ok(Value::i64(1))) => Verdict::Fires,
        Ok(Ok(Value::i64(n))) => Verdict::TemplateDefect(
            DefectKind::DidNotDiscriminate,
            format!("selected {n} rows where the constraint admits exactly 1"),
        ),
        Ok(Ok(other)) => {
            Verdict::TemplateDefect(DefectKind::NonCount, format!("entry returned {other:?}"))
        }
        Ok(Err(e)) => attribute(format!("{e:?}"), op),
        Err(payload) => {
            let msg = if let Some(p) = payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                p.message.clone()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                (*s).to_string()
            } else {
                "panic-opaque".to_string()
            };
            attribute(msg, op)
        }
    }
}

/// Split a refusal into "about the op" and "about my program".
///
/// Substring on the op's FQDN rather than on an error type name, deliberately: the diagnostic
/// wording belongs to whoever raises it, but a refusal that concerns this op has to say which op
/// — that is R29, a diagnostic that merely says "no" teaches nothing, and every rete refusal in
/// the tree names its head. If one ever does not, this returns `TemplateDefect` and the cell goes
/// loud instead of quietly joining the tally, which is the correct failure direction.
fn attribute(message: String, op: &str) -> Verdict {
    if message.contains(op) {
        Verdict::Refused(message)
    } else {
        Verdict::TemplateDefect(DefectKind::Unattributed, message)
    }
}

/// An i64 ordering comparison — the baseline row, reachable in BOTH positions.
const I64_GT: Cell = Cell {
    op: ":wat::rete::core::i64::>",
    field_ty: ":wat::core::i64",
    hit: "42",
    miss: "3",
    inline_rhs: "10",
    where_rhs: "10",
};

/// Keyword equality — reachable in a fence, NOT as an inline constraint. The asymmetry this whole
/// ledger exists because of.
const KEYWORD_EQ: Cell = Cell {
    op: ":wat::rete::core::keyword::=",
    field_ty: ":wat::core::keyword",
    hit: ":alpha",
    miss: ":beta",
    inline_rhs: ":alpha",
    where_rhs: ":alpha",
};

/// Report a cell's outcome with the SOURCE attached.
///
/// The doctrine this file deviates from buys `cargo wat`-runnability; printing the exact program
/// is how that is paid back. A failing cell must never make anyone reconstruct what was driven.
fn expect(cell: &Cell, site: CallSite, want: &Verdict) {
    let src = synth(cell, site);
    let got = drive(&src, cell.op);
    // `TemplateDefect` matches NOTHING — it is never an expected outcome, only ever a bug in the
    // cell's own program. Folding it into `Refused` here is precisely the collapse this file was
    // rewritten to prevent.
    let matches = matches!(
        (&got, want),
        (Verdict::Fires, Verdict::Fires) | (Verdict::Refused(_), Verdict::Refused(_))
    );
    assert!(
        matches,
        "CALIBRATION CELL MOVED — {op} @ {site}: expected {want:?}, got {got:?}\n\
         Nothing else this ledger reports can be trusted until this is understood.\n\
         (A `TemplateDefect` here means the synthesized program is malformed and the cell says \
         NOTHING about rete — fix the template, do not record a verdict.)\n\
         ─── the program driven ───\n{src}",
        op = cell.op,
        site = site.label(),
    );
}

/// ★★ THE CALIBRATION — four cells, two verdicts, every answer already known from the disk.
///
/// | cell | expected | why it is known |
/// |---|---|---|
/// | `i64::>` inline | FIRES | `tests/rete/probe_arc278_inline_constraint_per_type.wat` compiles, fires, prunes |
/// | `i64::>` fence | FIRES | the `where` family across the grid |
/// | `keyword::=` fence | FIRES | arc 109's NOTE, proven twice (`where-eq=1`) |
/// | `keyword::=` inline | REFUSED | arc 109's NOTE — `rete_type_segment_of` maps only the capital, uninhabitable `Keyword` |
///
/// Two of each verdict is the load-bearing part. A template that renders NOTHING would pass a
/// control made only of refusals; a template that never applies its constraint would pass one made
/// only of fires. Only a mixed control can fail in both directions.
#[test]
fn the_ledger_reproduces_four_known_cells_before_it_reports_an_unknown_one() {
    expect(&I64_GT, CallSite::InlineConstraint, &Verdict::Fires);
    expect(&I64_GT, CallSite::WhereFence, &Verdict::Fires);
    expect(&KEYWORD_EQ, CallSite::WhereFence, &Verdict::Fires);
    expect(&KEYWORD_EQ, CallSite::InlineConstraint, &Verdict::Refused(String::new()));
}

/// The asymmetry itself, stated as a property rather than as two independent cells.
///
/// This is what a per-ROW ledger would miss and why the unit is (row x call-site kind): the SAME
/// op, the SAME record, the SAME field, the SAME comparison — one position fires and the other
/// refuses. A ledger that recorded `keyword::=` as one row would have to pick one answer, and
/// either choice is a lie about half the surface.
#[test]
fn one_op_can_be_reachable_in_one_position_and_refused_in_another() {
    let fence = drive(&synth(&KEYWORD_EQ, CallSite::WhereFence), KEYWORD_EQ.op);
    let inline = drive(&synth(&KEYWORD_EQ, CallSite::InlineConstraint), KEYWORD_EQ.op);
    assert_eq!(fence, Verdict::Fires, "the fence position must be reachable");
    assert!(
        matches!(inline, Verdict::Refused(_)),
        "the inline position must be refused — if this ever starts firing, the asymmetry is \
         GONE and arc 109's NOTE plus this ledger's entire reason for existing are stale; got \
         {inline:?}"
    );
    // NON-VACUITY: the two positions must actually disagree. Without this the pair would pass
    // just as happily if the template made every cell refuse.
    assert_ne!(
        fence,
        inline,
        "the two positions must DISAGREE — that disagreement is the ledger's whole premise"
    );
}

/// ★★ THE ATTRIBUTION'S OWN GATE — a refusal that does not name the op is NOT a finding.
///
/// Without this, `attribute` could return `Refused` unconditionally and every calibration above
/// would still pass: three of the four cells are fires, and the fourth only checks that SOMETHING
/// refused. This is the row that makes the distinction load-bearing rather than decorative.
///
/// The break is chosen to have nothing whatever to do with the op: a binding reads a field the
/// record does not declare. The op in that program is `i64::>`, which is perfectly reachable —
/// so a ledger that counted this as `Refused` would record a false negative against a row it had
/// just proven reachable one cell earlier.
#[test]
fn a_refusal_that_does_not_name_the_op_is_a_template_defect_not_a_reachability_finding() {
    let good = synth(&I64_GT, CallSite::InlineConstraint);
    let broken = good.replace("(?k <- :k)", "(?k <- :nope)");
    assert_ne!(good, broken, "the break must actually change the program");

    match drive(&broken, I64_GT.op) {
        // EXACT, not a substring: the kind is the contract. `Unattributed` specifically — a
        // `DidNotDiscriminate` here would mean the broken field was silently ACCEPTED and the
        // rule merely stopped pruning, which is a completely different bug wearing the same
        // outer variant.
        Verdict::TemplateDefect(kind, _) => assert_eq!(
            kind,
            DefectKind::Unattributed,
            "the break was a field the record does not declare, so the refusal must be \
             unattributable to {op} — any other kind means the break did not land where intended",
            op = I64_GT.op,
        ),
        other => panic!(
            "a refusal unrelated to {op} must classify as TemplateDefect — recording it as a \
             reachability verdict would report a row DEAD that this same file proves reachable \
             in the very next cell; got {other:?}\n─── the program driven ───\n{broken}",
            op = I64_GT.op,
        ),
    }

    // THE CONTROL, and it is the half that can actually fail: the UNBROKEN twin must still be a
    // real answer. Without it this test would pass against an `attribute` that called everything
    // a TemplateDefect — which would silence the entire ledger while looking maximally careful.
    assert_eq!(
        drive(&good, I64_GT.op),
        Verdict::Fires,
        "the unbroken twin must still fire — otherwise this test proves only that the classifier \
         says TemplateDefect to everything"
    );
}

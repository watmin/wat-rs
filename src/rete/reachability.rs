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

use crate::ast::WatAST;
use crate::freeze::startup_from_source;
use crate::rete::vocabulary::{OpClass, ParamType, ReteOp, RETE_OPS};
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

/// The knobs one cell needs beyond what `RETE_OPS` already holds.
///
/// ⛔ **THIS DATA CANNOT BE DERIVED FROM `params`, AND THAT IS THE CENTRAL FACT OF THE GENERATOR.**
/// The row's types give the SHAPE of a call — how many operands, of what type. They do not give a
/// pair of facts the op tells APART, and telling them apart is the entire evidence: an op that is
/// reached and then admits everything has not been shown to run at all (`DidNotDiscriminate`).
///
/// Worked: for a binary op over `{a, b}` constrained against the literal `a` — `=` selects one,
/// `not=` selects the other, `>` selects one when `b > a`, and `<` selects **NONE**. Same types,
/// same arity, four different discriminating literals. So the generator is fed a small table of
/// literals per row and generates the PROGRAM; it does not guess the semantics.
///
/// The safety is that a wrong triple cannot pass quietly — it lands as `DidNotDiscriminate` and
/// fails loudly, so this table is machine-checked rather than trusted.
struct Cell {
    /// The rete-surface FQDN under test.
    op: &'static str,
    /// How many operands the row declares (`ReteOp::params.len()`), so a unary op renders
    /// `(OP :v)` and a binary one `(OP :v RHS)`.
    arity: usize,
    /// wat type of the discriminating field.
    field_ty: &'static str,
    /// Literal for the fact that SHOULD survive the constraint.
    hit: &'static str,
    /// Literal for the fact that should NOT.
    miss: &'static str,
    /// The right-hand operand, written identically in both positions — ignored when `arity == 1`.
    ///
    /// It is the same TEXT in both, deliberately, because the two positions READ it differently
    /// and that difference is a finding rather than something to paper over: inline, a bare
    /// keyword is a field reference and never a keyword value (`matcher.rs`'s `ast_literal_value`);
    /// inside a fence there is no such grammar. Writing one spelling and letting each position do
    /// what it really does is what surfaces that.
    rhs: &'static str,
}

/// Build the complete program for one cell.
///
/// Both arms differ ONLY in the condition vector — same records, same facts, same query, same
/// entry point. That is deliberate: it means a difference in outcome between two cells of the
/// same row is attributable to the POSITION and to nothing else.
fn synth(cell: &Cell, site: CallSite) -> String {
    let (inline_call, fence_call) = if cell.arity <= 1 {
        (format!("({} :v)", cell.op), format!("({} ?v)", cell.op))
    } else {
        (
            format!("({} :v {})", cell.op, cell.rhs),
            format!("({} ?v {})", cell.op, cell.rhs),
        )
    };
    let condition = match site {
        CallSite::InlineConstraint => format!("(:probe::In (?k <- :k) {inline_call})"),
        CallSite::WhereFence => format!(
            "(:probe::In (?k <- :k) (?v <- :v))\n   (:wat::rete::where {fence_call})"
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
    if message.contains(op) || message.contains(&as_diagnostics_spell_it(op)) {
        Verdict::Refused(message)
    } else {
        Verdict::TemplateDefect(DefectKind::Unattributed, message)
    }
}

/// The op's name as a DIAGNOSTIC writes it — asked of the renderer, never reconstructed.
///
/// ⛔ **THIS IS NOT A SPELLING CONVERTER AND MUST NOT BECOME ONE.** wat is mid-migration toward
/// Clojure/EDN-compliant syntax: `:wat::core::+` is written `wat.core/+` there, and heads are
/// moving from keywords to symbols. Diagnostics already render the EDN spelling — a
/// `MalformedClause` for `:wat::rete::core::not` names it `:wat.rete.core/not` — while `RETE_OPS`
/// holds the `::` form. So a refusal about an op can arrive under a name the table does not use.
///
/// The first draft compared only the `::` form and therefore filed real refusals as
/// `TemplateDefect`: SEVEN cells, every one of them a genuine `MalformedClause` naming its op in
/// the other spelling. Hand-rolling the `::`->`.`/`/` transform here would have fixed those cells
/// and planted a SECOND encoding of the naming rule, to go stale at the exact moment the migration
/// lands — the `solvere` duplication class this arc keeps pulling out.
///
/// Instead this asks `validate::render_form` — the very function the diagnostics use — what the
/// name looks like. It is correct by construction today, and it follows the migration for free:
/// when heads become symbols, the renderer changes and this changes with it, with nothing here
/// to update.
fn as_diagnostics_spell_it(op: &str) -> String {
    crate::rete::validate::render_form(&WatAST::Keyword(op.to_string(), crate::rust_caller_span!()))
}

/// An i64 ordering comparison — the baseline row, reachable in BOTH positions.
const I64_GT: Cell = Cell {
    op: ":wat::rete::core::i64::>",
    arity: 2,
    field_ty: ":wat::core::i64",
    hit: "42",
    miss: "3",
    rhs: "10",
};

/// Keyword equality — reachable in a fence, NOT as an inline constraint. The asymmetry this whole
/// ledger exists because of.
const KEYWORD_EQ: Cell = Cell {
    op: ":wat::rete::core::keyword::=",
    arity: 2,
    field_ty: ":wat::core::keyword",
    hit: ":alpha",
    miss: ":beta",
    rhs: ":alpha",
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

// ─── The generator ─────────────────────────────────────────────────────────────────────────────

/// The operand table — one entry per Bool-returning `Alias` row.
///
/// This is the ONLY hand-written data in the sweep; everything else (arity, the op's name, which
/// rows must appear here at all) comes from `RETE_OPS` itself, so the table cannot silently fall
/// behind the vocabulary: a row minted without an entry is a RED BUILD, not a row nobody notices.
/// That is the same shape as `every_rete_row_is_total` and for the same reason — a count cannot
/// tell "+1 new, -1 fixed" from "nothing happened", so this names the offender instead.
///
/// `None` means "no entry", which the sweep treats as a failure. There is deliberately no
/// "skip this row" value: an unclassifiable row must be argued in prose at the exclusion list
/// below, where a reader can disagree with it.
fn operands_for(rete_name: &'static str) -> Option<Cell> {
    let (arity, field_ty, hit, miss, rhs) = match rete_name {
        // i64 — the baseline. Note `<` needs its hit/miss SWAPPED relative to `>` against the
        // same literal, which is the whole argument for a per-row table.
        ":wat::rete::core::i64::>" => (2, ":wat::core::i64", "42", "3", "10"),
        ":wat::rete::core::i64::<" => (2, ":wat::core::i64", "3", "42", "10"),
        ":wat::rete::core::i64::>=" => (2, ":wat::core::i64", "10", "3", "10"),
        ":wat::rete::core::i64::<=" => (2, ":wat::core::i64", "10", "42", "10"),
        ":wat::rete::core::i64::=" => (2, ":wat::core::i64", "10", "3", "10"),
        ":wat::rete::core::i64::not=" => (2, ":wat::core::i64", "3", "10", "10"),

        // f64 — `>=`/`<=` pin the BOUNDARY (hit == rhs), so an implementation that dropped the
        // `=` half would go red here rather than passing on the strict half alone.
        ":wat::rete::core::f64::>" => (2, ":wat::core::f64", "42.0", "3.0", "10.0"),
        ":wat::rete::core::f64::<" => (2, ":wat::core::f64", "3.0", "42.0", "10.0"),
        ":wat::rete::core::f64::>=" => (2, ":wat::core::f64", "10.0", "3.0", "10.0"),
        ":wat::rete::core::f64::<=" => (2, ":wat::core::f64", "10.0", "42.0", "10.0"),
        ":wat::rete::core::f64::=" => (2, ":wat::core::f64", "10.0", "3.0", "10.0"),
        ":wat::rete::core::f64::not=" => (2, ":wat::core::f64", "3.0", "10.0", "10.0"),

        // String — the three predicates use a needle that is a strict INFIX/PREFIX/SUFFIX of the
        // hit and absent from the miss, so each one tests its own half rather than plain equality.
        ":wat::rete::core::String/starts-with?" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"al\""),
        ":wat::rete::core::String/ends-with?" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"ha\""),
        ":wat::rete::core::String/contains?" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"lph\""),
        ":wat::rete::core::String/empty?" => (1, ":wat::core::String", "\"\"", "\"x\"", ""),
        ":wat::rete::core::string::=" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"alpha\""),
        ":wat::rete::core::string::not=" => (2, ":wat::core::String", "\"beta\"", "\"alpha\"", "\"alpha\""),

        // bool
        ":wat::rete::core::not" => (1, ":wat::core::bool", "false", "true", ""),
        ":wat::rete::core::bool::=" => (2, ":wat::core::bool", "true", "false", "true"),
        ":wat::rete::core::bool::not=" => (2, ":wat::core::bool", "false", "true", "true"),

        // keyword — `=` is the motivating asymmetry; `not=` is one of the three rows that appear
        // NOWHERE in the 1569-file corpus, so its cells are the first evidence it has ever had.
        ":wat::rete::core::keyword::=" => (2, ":wat::core::keyword", ":alpha", ":beta", ":alpha"),
        ":wat::rete::core::keyword::not=" => (2, ":wat::core::keyword", ":beta", ":alpha", ":alpha"),

        // Containers — a parametric field, so these also test that the template survives a
        // non-scalar declaration.
        ":wat::rete::core::PersistentVector/contains?" => (
            2,
            "(:wat::core::PersistentVector :- [:wat::core::i64])",
            "(:wat::core::PersistentVector 1 2)",
            "(:wat::core::PersistentVector 9)",
            "1",
        ),
        ":wat::rete::core::PersistentMap/contains-key?" => (
            2,
            "(:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])",
            "(:wat::core::PersistentMap \"a\" 1)",
            "(:wat::core::PersistentMap \"z\" 1)",
            "\"a\"",
        ),

        _ => return None,
    };
    Some(Cell { op: rete_name, arity, field_ty, hit, miss, rhs })
}

/// The rows deliberately NOT in the operand table, each with the reason a reader can argue with.
///
/// An exclusion is a claim that a cell cannot be written, which is exactly the kind of claim this
/// arc has been wrong about twice (see the breadcrumb: "do not trust a grep that found nothing").
/// So each one names what would refute it.
const NOT_YET_GENERABLE: &[(&str, &str)] = &[(
    ":wat::rete::holon::presence?",
    "takes TWO `:wat::holon::HolonAST` operands; a holon has no literal spelling, so the second \
     operand cannot be written as a constant the way every scalar row's can. REFUTED BY: any rule \
     that reaches this op with a constructed holon on both sides — at which point it belongs in \
     the table above, not here.",
)];

/// ★★ THE SWEEP — every Bool-returning `Alias` row, in every modelled position.
///
/// These 26 rows are the block that is directly constraint-shaped: they return `bool`, so they can
/// be written where a constraint goes without being wrapped in a comparison first. The other 48
/// rows (non-`bool` `Alias`, all of `Fallback`, and the param-less `Form`/`Redispatch`) need a
/// wrapping or a bespoke shape and are a separate strike — deliberately not guessed at here, since
/// an un-calibrated position is how a template manufactures a column of false findings.
///
/// **What this gate makes impossible:** minting a rete row that no user can reach. `RETE_OPS`
/// gates purity, totality, arity and type; none of them asks whether the op can be CALLED. A new
/// Bool-returning `Alias` row with no operand entry fails here by name.
///
/// The full matrix prints on every run, pass or fail. A ledger whose output is only visible when
/// it breaks is a ledger nobody reads.
#[test]
fn every_bool_returning_alias_row_has_a_verdict_in_every_modelled_position() {
    let rows: Vec<&ReteOp> = RETE_OPS
        .iter()
        .filter(|o| o.class == OpClass::Alias && matches!(o.ret, ParamType::Bool))
        .collect();

    // NON-VACUITY: a filter that selects nothing finds nothing wrong. This number is asserted as
    // a FLOOR rather than frozen exactly — new rows are expected, a collapse to zero is not.
    assert!(
        rows.len() >= 26,
        "the Bool-returning Alias block looks empty or renamed ({} rows) — this sweep would pass \
         vacuously",
        rows.len()
    );

    let mut unclassified: Vec<&str> = Vec::new();
    let mut defects: Vec<String> = Vec::new();
    let mut matrix: Vec<String> = Vec::new();

    for row in &rows {
        let Some(cell) = operands_for(row.rete_name) else {
            if let Some((_, why)) = NOT_YET_GENERABLE.iter().find(|(n, _)| *n == row.rete_name) {
                matrix.push(format!("{:<46} {:>17}  {}", row.rete_name, "NOT-GENERABLE", why));
            } else {
                unclassified.push(row.rete_name);
            }
            continue;
        };
        // The row's own arity is the authority; a table entry that disagrees is a table bug, and
        // silently trusting either one would let the two drift.
        assert_eq!(
            cell.arity,
            row.params.len(),
            "operand table says arity {} for {} but the row declares {} params",
            cell.arity,
            row.rete_name,
            row.params.len()
        );

        let mut verdicts: Vec<String> = Vec::new();
        for site in [CallSite::InlineConstraint, CallSite::WhereFence] {
            let src = synth(&cell, site);
            match drive(&src, cell.op) {
                Verdict::Fires => verdicts.push(format!("{}=FIRES", site.label())),
                Verdict::Refused(_) => verdicts.push(format!("{}=REFUSED", site.label())),
                Verdict::TemplateDefect(kind, detail) => {
                    verdicts.push(format!("{}=DEFECT", site.label()));
                    defects.push(format!(
                        "  {} @ {}: {kind:?} — {detail}\n─── the program driven ───\n{src}",
                        cell.op,
                        site.label()
                    ));
                }
            }
        }
        matrix.push(format!("{:<46} {}", row.rete_name, verdicts.join("  ")));
    }

    println!("\n─── RETE_OPS reachability, Bool-returning Alias rows ───");
    for line in &matrix {
        println!("{line}");
    }
    println!("─── {} rows ───\n", rows.len());

    assert!(
        unclassified.is_empty(),
        "these rete rows have NO reachability verdict — a row that no cell exercises is a row \
         nobody has shown a user can reach. Add an operand entry, or an argued exclusion in \
         NOT_YET_GENERABLE: {unclassified:#?}"
    );
    assert!(
        defects.is_empty(),
        "the GENERATOR is wrong for these cells, not rete — a TemplateDefect says the synthesized \
         program is malformed, so no verdict may be recorded from it:\n{}",
        defects.join("\n\n")
    );
}

// ─── The third axis: SPELLING ──────────────────────────────────────────────────────────────────

/// How the op's head is written. The migration axis.
///
/// wat is grinding toward Clojure/EDN-compliant SYNTAX (not a Clojure implementation — the
/// spelling): `:wat::core::+` is `wat.core/+` there, and heads are moving from keywords to
/// symbols. rete's `:when` DSL is believed to accept only the `::` form today.
///
/// This belongs in the reachability ledger for exactly the reason the call-site axis does:
/// reachability is not a property of the ROW. An op reachable as `:wat::rete::core::>` and
/// refused as `wat.rete.core/>` is the same defect shape as one reachable in a fence and refused
/// inline — and once the flip lands, the column that is red today becomes the column that must be
/// green, so measuring it now turns the migration's progress into something a gate can read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spelling {
    /// `:wat::rete::core::>` — what `RETE_OPS` holds and what the DSL is believed to require.
    RustyKeyword,
    /// `:wat.rete.core/>` — EDN-compliant keyword; already what diagnostics PRINT.
    DottedKeyword,
    /// `wat.rete.core/>` — a bare SYMBOL head, the shape after the keyword->symbol flip.
    DottedSymbol,
}

impl Spelling {
    fn label(self) -> &'static str {
        match self {
            Spelling::RustyKeyword => "::keyword",
            Spelling::DottedKeyword => ":dotted/kw",
            Spelling::DottedSymbol => "dotted/sym",
        }
    }

    /// Render an op's head in this spelling.
    ///
    /// The dotted forms come from `as_diagnostics_spell_it` — the renderer — rather than from a
    /// local `replace("::", ".")`, so this file holds NO copy of the naming rule. `DottedSymbol`
    /// is that same string minus the leading `:`, which is the whole difference between a keyword
    /// and a symbol head.
    fn render(self, op: &str) -> String {
        match self {
            Spelling::RustyKeyword => op.to_string(),
            Spelling::DottedKeyword => as_diagnostics_spell_it(op),
            Spelling::DottedSymbol => {
                let kw = as_diagnostics_spell_it(op);
                kw.strip_prefix(':').unwrap_or(&kw).to_string()
            }
        }
    }
}

/// ★★ THE MIGRATION BASELINE — which head spellings rete's `:when` DSL accepts today.
///
/// This test asserts NOTHING about which spellings ought to work; it pins what IS, so the flip has
/// a before-picture instead of a memory. It fails only if the `::` form — the one the whole corpus
/// is written in — stops working, which would be a live regression rather than a migration step.
///
/// The other two columns print their verdict and are deliberately un-asserted: they are expected
/// to be refused now and expected to be required later, so hard-coding either answer would make
/// this test a thing to delete at the flip rather than the thing that MEASURES the flip.
#[test]
fn the_head_spellings_rete_accepts_today_are_recorded_for_the_edn_migration() {
    let cell = operands_for(":wat::rete::core::i64::>").expect("the baseline row must be in the table");
    let mut lines: Vec<String> = Vec::new();
    let mut rusty_ok = 0usize;

    for spelling in [Spelling::RustyKeyword, Spelling::DottedKeyword, Spelling::DottedSymbol] {
        let head = spelling.render(cell.op);
        for site in [CallSite::InlineConstraint, CallSite::WhereFence] {
            // Surgical: synthesize in the canonical spelling, then rewrite ONLY the head. Every
            // other byte of the program is identical across the three columns, so a difference
            // is attributable to the spelling and to nothing else.
            let src = synth(&cell, site).replace(cell.op, &head);
            let verdict = match drive(&src, cell.op) {
                Verdict::Fires => "FIRES",
                Verdict::Refused(_) => "REFUSED",
                Verdict::TemplateDefect(..) => "REFUSED(unattributed)",
            };
            if spelling == Spelling::RustyKeyword && verdict == "FIRES" {
                rusty_ok += 1;
            }
            lines.push(format!("{:<12} {:<18} {}", spelling.label(), site.label(), verdict));
        }
    }

    println!("\n─── head spelling x call site, `:wat::rete::core::i64::>` ───");
    for l in &lines {
        println!("{l}");
    }
    println!("─── EDN-migration baseline; only the `::` column is asserted ───\n");

    assert_eq!(
        rusty_ok, 2,
        "the `::` spelling must work in BOTH positions — the entire corpus is written in it, so \
         this going red is a live regression, not a migration step"
    );

    // ⛔ THE CONTROL, and the surprising column is why it exists. `dotted/sym` — a BARE SYMBOL
    // head, the shape rete does not officially accept — comes back FIRES inside a `where` fence.
    // That reads like a gift for the migration, and a green that surprising is exactly the kind
    // this arc has twice reported without checking. So: a symbol head naming an op that DOES NOT
    // EXIST must be refused. If a nonsense head also "fires", then symbol heads are not being
    // dispatched at all — something else is satisfying the fence — and the whole column means
    // nothing.
    let nonsense = synth(&cell, CallSite::WhereFence)
        .replace(cell.op, "wat.rete.core/no-such-op-exists");
    let verdict = drive(&nonsense, cell.op);
    assert!(
        !matches!(verdict, Verdict::Fires),
        "a SYMBOL head naming a nonexistent op fired — so the `dotted/sym` column above is not \
         evidence that symbol heads dispatch, and no claim may be made from it; got {verdict:?}"
    );

    // ⛔⛔ THE SECOND CONTROL, and it decides what the first one MEANS. A symbol head dispatching
    // is only good news if it dispatches to the RETE row. If `wat.core/>` — the CORE op, which
    // Law A refuses inside a fence in every other spelling — also fires, then symbol heads are
    // not a migration gift at all: they are a BYPASS of the fence whose entire job is
    // *"the rete query language may only be composed from rete primitives"* (`purity.rs`'s
    // `Axis::RetePrimitive`, which refuses `:wat::core::>` precisely because being pure,
    // deterministic and total does not make an op rete).
    let core_spelled = synth(&cell, CallSite::WhereFence).replace(cell.op, "wat.core/>");
    let core_verdict = drive(&core_spelled, cell.op);
    assert!(
        !matches!(core_verdict, Verdict::Fires),
        "LAW A IS BYPASSED BY SYMBOL SPELLING — `wat.core/>` is a CORE op and fired inside a \
         `where` fence, where `:wat::core::>` is refused. The fence checks the head it was \
         taught to check and this spelling walks past it, so the `dotted/sym` column is a HOLE \
         and not readiness; got {core_verdict:?}\n─── the program driven ───\n{core_spelled}"
    );
}

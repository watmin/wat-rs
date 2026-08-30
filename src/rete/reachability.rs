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
use crate::rete::vocabulary::{ReteOp, RETE_OPS};
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
#[derive(Debug, Clone, PartialEq, Eq)]
enum Verdict {
    /// A rule was written at this position, compiled, fired, and selected the row it should.
    Fires,
    /// The load path refused it AND the diagnostic named the op under test. A real answer: a
    /// user cannot reach this row here. The message is carried because the REASON is the finding —
    /// refused for "no comparator for this type" is a different defect from "that is not a rete
    /// head".
    Refused(String),
    /// It compiled, fired, and matched NOTHING — while the SAME cell in another position
    /// discriminates correctly. That cross-position control is what makes this a finding about
    /// rete rather than a bad operand: the operands are proven good by the position that works.
    ///
    /// It is deliberately NOT folded into `Refused`. A refusal teaches the user; this is accepted,
    /// runs, and is unsatisfiable — a silent wrong answer, which is the class this arc exists to
    /// eliminate and the one a differential cannot see.
    MatchesNothing,
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
    /// It compiled and FIRED and selected NOTHING. Ambiguous ON ITS OWN — either the cell's
    /// expected value is wrong, or the position is broken — so the sweep adjudicates it against
    /// the SAME cell in the other position (see `Verdict::MatchesNothing`).
    MatchedNothing,
    /// It compiled and FIRED and selected MORE than the constraint admits. Never ambiguous: the
    /// constraint is not constraining, so the cell is not evidence of anything and counting it as
    /// `Fires` would be the ledger's worst false positive.
    MatchedTooMany,
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
    /// The constraint expression, with `{f}` wherever the discriminating field is referenced.
    ///
    /// This is the ONE representation of what a cell writes. `{f}` renders as `:v` inline (where a
    /// bare keyword is a field reference) and `?v` in a fence (where it is the bound variable) —
    /// the same text, read differently by each position, which is what makes the two columns
    /// comparable.
    ///
    /// Most rows get this from [`uniform_call`], which builds the `(op operands…)` shape from the
    /// row's arity plus a wrap. Special forms cannot use that shape at all — `and` takes nested
    /// predicates, `foldl` takes a `fn` — so they state their expression directly. One
    /// representation, two builders; a builder is not a second source of truth.
    expr: String,
    /// True when `expr` was stated VERBATIM (from `special_for`) rather than built by
    /// [`uniform_call`] from the row's arity.
    ///
    /// This is what the arity cross-check keys on, and the discriminator matters: the check used
    /// to key on `Form | Redispatch`, which was a PROXY for "states its own expression" and stopped
    /// being true the moment an `Alias` row needed a bespoke shape (the keyword converters, whose
    /// operands must be wrapped because a keyword literal is unwritable in operand position). A
    /// verbatim cell's arity drives NOTHING, so there is nothing to cross-check — and saying that
    /// directly is honest where the class proxy was merely correct-so-far.
    expr_is_verbatim: bool,
    /// Extra top-level declarations this cell needs before the records — an enum for `enum::=`,
    /// say. Empty for almost every row.
    extra: &'static str,
}

/// Build the complete program for one cell.
///
/// Both arms differ ONLY in the condition vector — same records, same facts, same query, same
/// entry point. That is deliberate: it means a difference in outcome between two cells of the
/// same row is attributable to the POSITION and to nothing else.
fn synth(cell: &Cell, site: CallSite) -> String {
    let condition = match site {
        CallSite::InlineConstraint => {
            format!("(:probe::In (?k <- :k) {})", cell.expr.replace("{f}", ":v"))
        }
        CallSite::WhereFence => format!(
            "(:probe::In (?k <- :k) (?v <- :v))\n   (:wat::rete::where {})",
            cell.expr.replace("{f}", "?v")
        ),
    };
    format!(
        r#"{extra}(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- {field_ty}])
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
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v {hit})) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v {miss})) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#,
        extra = cell.extra,
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
        Ok(Ok(Value::i64(0))) => Verdict::TemplateDefect(
            DefectKind::MatchedNothing,
            "selected 0 rows where the constraint admits exactly 1".to_string(),
        ),
        Ok(Ok(Value::i64(n))) => Verdict::TemplateDefect(
            DefectKind::MatchedTooMany,
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
fn i64_gt() -> Cell {
    operands_for(":wat::rete::core::i64::>").expect("the baseline row must be in the table")
}

/// Keyword equality — the asymmetry this whole ledger exists because of, and CLOSED 2026-08-28.
/// It is kept as a calibration cell precisely because its expected verdict CHANGED: it is the
/// worked example of the ledger noticing a surface move rather than a driver breaking.
fn keyword_eq() -> Cell {
    operands_for(":wat::rete::core::keyword::=").expect("the keyword row must be in the table")
}

/// ★ THE DURABLE REFUSAL — Law A: the rete query language is composed from RETE primitives, so a
/// core-spelled head is refused in EVERY position, by design, permanently.
///
/// ⚠ **This cell exists because the calibration ran out of refusals.** Until 2026-08-28 the
/// mixed control was `keyword::=` — reachable in a fence, refused inline. That asymmetry is now
/// gone (deliberately: the surface admits the constant), and with it went the only `Refused`
/// verdict the calibration had. A control made only of fires is passed by a driver that never
/// applies its constraint, which is exactly the failure this file's own header warns about — so
/// the refusal had to be re-sourced from something that cannot become reachable. Law A is that
/// thing: a `:wat::core::` head being refused is not a gap, it is the surface's founding rule.
fn law_a_core_head() -> Cell {
    Cell {
        op: ":wat::core::>",
        arity: 0,
        expr_is_verbatim: true,
        field_ty: ":wat::core::i64",
        hit: "42",
        miss: "3",
        extra: "",
        expr: "(:wat::core::> {f} 10)".to_string(),
    }
}

/// Report a cell's outcome with the SOURCE attached.
///
/// The doctrine this file deviates from buys `cargo wat`-runnability; printing the exact program
/// is how that is paid back. A failing cell must never make anyone reconstruct what was driven.
fn expect(cell: &Cell, site: CallSite, want: &Verdict) {
    let src = synth(cell, site);
    let mut got = drive(&src, cell.op);
    // The SAME cross-position adjudication the sweep uses, and for the same reason: the expression
    // text is identical in both positions, so a cell that fires anywhere is valid wat and a
    // refusal elsewhere is about the POSITION whatever the diagnostic happened to name.
    //
    // This became load-bearing on 2026-08-28. The keyword-inline cell used to refuse with
    // `ConstraintTypeNotComparable` — which NAMES the op — because `rete_type_segment_of` mapped
    // only the uninhabitable capital `Keyword`. That is fixed, so the cell now refuses one step
    // later with `UnknownField`: `:alpha` in operand position is read as a FIELD REFERENCE
    // (`matcher.rs`'s `ast_literal_value`), and that error names the field, not the op. Still a
    // genuine position refusal; only the reason moved.
    if matches!(got, Verdict::TemplateDefect(DefectKind::Unattributed, _))
        && matches!(drive(&synth(cell, CallSite::WhereFence), cell.op), Verdict::Fires)
    {
        if let Verdict::TemplateDefect(_, m) = got {
            got = Verdict::Refused(m);
        }
    }
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
/// | `keyword::=` inline | FIRES | **since 2026-08-28** — see below |
/// | `:wat::core::>` inline | REFUSED | Law A: the rete surface admits only rete heads |
/// | `:wat::core::>` fence | REFUSED | Law A again — refused in every position, by design |
///
/// ⚠ **THE KEYWORD ROW FLIPPED FROM `REFUSED` TO `FIRES`, AND THAT IS THE CALIBRATION WORKING.**
/// Its verdict has now moved twice. It first refused with `ConstraintTypeNotComparable` (the
/// keyword type-map recognised only the uninhabitable capital `Keyword`), then one step later with
/// `UnknownField` once that was fixed, and now not at all: a keyword operand naming no declared
/// field is read as the CONSTANT it is, the same rule the nested-operand path always used. A
/// calibration cell whose answer is allowed to change — deliberately, in one commit, with the
/// reason written down — is the instrument noticing a surface move. One that is never allowed to
/// change is a golden nobody re-reads.
///
/// ⛔ **AND THAT FLIP COST THE CALIBRATION ITS ONLY REFUSAL, WHICH IS WHY LAW A IS NOW HERE.**
/// Two of each verdict is the load-bearing part: a template that renders NOTHING passes a control
/// made only of refusals, and one that never applies its constraint passes a control made only of
/// fires. With `keyword::=` reachable, every remaining cell was a fire — so the control had gone
/// one-directional without a single assertion changing. The refusal is re-sourced from Law A,
/// which cannot become reachable without the rete surface ceasing to be one.
#[test]
fn the_ledger_reproduces_four_known_cells_before_it_reports_an_unknown_one() {
    expect(&i64_gt(), CallSite::InlineConstraint, &Verdict::Fires);
    expect(&i64_gt(), CallSite::WhereFence, &Verdict::Fires);
    expect(&keyword_eq(), CallSite::WhereFence, &Verdict::Fires);
    expect(&keyword_eq(), CallSite::InlineConstraint, &Verdict::Fires);
    expect(&law_a_core_head(), CallSite::InlineConstraint, &Verdict::Refused(String::new()));
    expect(&law_a_core_head(), CallSite::WhereFence, &Verdict::Refused(String::new()));
}

/// ★★ THE ASYMMETRY THIS LEDGER WAS BUILT FOR IS CLOSED — and the unit stays (row x position).
///
/// ⚠ **THIS TEST USED TO ASSERT THE OPPOSITE, AND IT SAID SO IN ITS OWN FAILURE MESSAGE**: *"if
/// this ever starts firing, the asymmetry is GONE and arc 109's NOTE plus this ledger's entire
/// reason for existing are stale."* It started firing on 2026-08-28, on purpose, and it took the
/// floor red — which is the alarm behaving exactly as designed. The asymmetry was a DEFECT, and
/// the whole point of the ledger was to find and close it. A gate that pins a defect must be
/// rewritten when the defect dies, and rewritten deliberately: quietly deleting it would erase the
/// evidence that the surface ever had the hole.
///
/// **The unit is still (row x call-site kind), and the reason is stronger than before, not weaker.**
/// The two positions are DIFFERENT MACHINERY — an inline constraint compiles through
/// `compiled_cond` into an alpha's op list; a fence lowers through `expr_ir` behind
/// `:wat::rete::where`. They agreed on nothing for the life of the engine and now agree on every
/// generable row. That agreement is a PROPERTY THE SWEEP HOLDS, not a fact about the language: it
/// can regress at any time, and only a per-position drive would see it.
///
/// **What is asserted here now** is the thing that no longer follows from the engine and so must
/// be checked directly: that the driver actually renders two DIFFERENT programs. While the two
/// positions disagreed, a driver that rendered one program twice was caught instantly by the
/// verdicts diverging. Now that every verdict agrees, that bug would be invisible — a
/// position-blind driver would report a perfect ledger. So the rendering is pinned structurally.
#[test]
fn the_two_positions_render_differently_and_now_agree() {
    let cell = keyword_eq();
    let fence_src = synth(&cell, CallSite::WhereFence);
    let inline_src = synth(&cell, CallSite::InlineConstraint);

    // ⛔ THE DRIVER CONTROL, and it is load-bearing precisely because the engine no longer
    // distinguishes these. A driver that rendered the same program for both positions would now
    // agree with itself on all 79 rows and look like a clean sweep.
    assert_ne!(
        fence_src, inline_src,
        "the two call sites must render DIFFERENT programs — with the verdicts no longer \
         diverging, this is the only thing left that can catch a position-blind driver"
    );
    // Counted, not `contains`-ed. The count is exact and strictly stronger — "exactly one fence"
    // rules out a rendering that opened two — and it sidesteps the loose-string-assert rune
    // entirely: a substring test would have needed an exemption, while a number needs none. A
    // whole-program golden would be the wrong tool here for the reason the rubric names: it would
    // go stale on any edit to the synthesis template, which is not what this is measuring.
    assert_eq!(
        fence_src.matches(":wat::rete::where").count(),
        1,
        "the fence rendering must place the predicate behind exactly one fence"
    );
    assert_eq!(
        inline_src.matches(":wat::rete::where").count(),
        0,
        "the inline rendering must NOT use the fence — otherwise it measures the fence twice"
    );

    // And the verdicts: equal, and equal to FIRES. `assert_eq` on the pair rather than two
    // separate `Fires` checks, so a future divergence names itself as a divergence.
    let fence = drive(&fence_src, cell.op);
    let inline = drive(&inline_src, cell.op);
    assert_eq!(
        fence, inline,
        "the same comparison must answer the same way in both positions; they disagreed for the \
         life of the engine and that was fix-list F's whole family"
    );
    assert_eq!(fence, Verdict::Fires, "and the answer they agree on must be FIRES");
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
    let good = synth(&i64_gt(), CallSite::InlineConstraint);
    let broken = good.replace("(?k <- :k)", "(?k <- :nope)");
    assert_ne!(good, broken, "the break must actually change the program");

    match drive(&broken, i64_gt().op) {
        // EXACT, not a substring: the kind is the contract. `Unattributed` specifically — a
        // `DidNotDiscriminate` here would mean the broken field was silently ACCEPTED and the
        // rule merely stopped pruning, which is a completely different bug wearing the same
        // outer variant.
        Verdict::TemplateDefect(kind, _) => assert_eq!(
            kind,
            DefectKind::Unattributed,
            "the break was a field the record does not declare, so the refusal must be \
             unattributable to {op} — any other kind means the break did not land where intended",
            op = i64_gt().op,
        ),
        other => panic!(
            "a refusal unrelated to {op} must classify as TemplateDefect — recording it as a \
             reachability verdict would report a row DEAD that this same file proves reachable \
             in the very next cell; got {other:?}\n─── the program driven ───\n{broken}",
            op = i64_gt().op,
        ),
    }

    // THE CONTROL, and it is the half that can actually fail: the UNBROKEN twin must still be a
    // real answer. Without it this test would pass against an `attribute` that called everything
    // a TemplateDefect — which would silence the entire ledger while looking maximally careful.
    assert_eq!(
        drive(&good, i64_gt().op),
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
/// Build the ordinary `(op {f} operands…)` expression, optionally wrapped in a comparator.
///
/// The shorthand every row that IS a plain call uses. Unary rows render `(op {f})`; everything
/// else `(op {f} rest)`, where `rest` is the operand text after the field — which is also how a
/// `Fallback` row's mandatory `:undefined <value>` marker pair is supplied.
fn uniform_call(op: &str, arity: usize, rest: &str, wrap: Option<(&str, &str)>) -> String {
    let call =
        if arity <= 1 { format!("({op} {{f}})") } else { format!("({op} {{f}} {rest})") };
    match wrap {
        None => call,
        Some((cmp, expected)) => format!("({cmp} {call} {expected})"),
    }
}


/// SPECIAL FORMS and re-dispatched functions — the rows `uniform_call` cannot build.
///
/// `Form` members are genuine special forms: `and` takes nested predicates, `let` binds, `cond`
/// has arms. `Redispatch` members are plain functions whose type is polymorphic over the CONTAINER
/// constructor, and several are constructors themselves, so the field has to be built INTO a
/// larger expression rather than passed as an operand.
///
/// ⛔ **Every expression here still has to DISCRIMINATE** — the hit fact passes and the miss fact
/// does not — which for a constructor means threading `{f}` inside the thing being constructed and
/// then asking a question about it. A cell that merely mentions the op without letting the field
/// change the answer would report `Fires` while proving nothing, and `MatchedTooMany` is what
/// catches that.
fn special_for(rete_name: &str) -> Option<(&'static str, &'static str, &'static str, &'static str, &'static str)> {
    let t = match rete_name {
        ":wat::rete::core::and" => (":wat::core::i64", "10", "1", "(:wat::rete::core::and (:wat::rete::core::i64::> {f} 5) (:wat::rete::core::i64::< {f} 20))", ""),
        ":wat::rete::core::or" => (":wat::core::i64", "1", "10", "(:wat::rete::core::or (:wat::rete::core::i64::> {f} 100) (:wat::rete::core::i64::< {f} 5))", ""),
        ":wat::rete::core::if" => (":wat::core::i64", "10", "1", "(:wat::rete::core::i64::= (:wat::rete::core::if (:wat::rete::core::i64::> {f} 5) 1 0) 1)", ""),
        ":wat::rete::core::let" => (":wat::core::i64", "10", "1", "(:wat::rete::core::let [x {f}] (:wat::rete::core::i64::> x 5))", ""),
        ":wat::rete::core::cond" => (":wat::core::i64", "10", "1", "(:wat::rete::core::cond ((:wat::rete::core::i64::> {f} 5) true) (:else false))", ""),
        ":wat::rete::core::match" => (":probe::E", ":probe::E::A", ":probe::E::B", "(:wat::rete::core::match {f} (:probe::E::A true) (:probe::E::B false))", "(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)\n\n"),
        ":wat::rete::core::fn" => ("(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 1 2)", "(:wat::core::PersistentVector 9)", "(:wat::rete::core::i64::= (:wat::rete::core::foldl (:wat::rete::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ acc x :undefined 0)) 0 {f}) 3)", ""),
        ":wat::rete::core::enum::=" => (":probe::E", ":probe::E::A", ":probe::E::B", "(:wat::rete::core::enum::= {f} :probe::E::A)", "(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)\n\n"),
        ":wat::rete::core::enum::not=" => (":probe::E", ":probe::E::B", ":probe::E::A", "(:wat::rete::core::enum::not= {f} :probe::E::A)", "(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)\n\n"),
        ":wat::rete::core::PersistentVector" => (":wat::core::i64", "7", "9", "(:wat::rete::core::PersistentVector/contains? (:wat::rete::core::PersistentVector {f} 99) 7)", ""),
        ":wat::rete::core::Vector" => (":wat::core::i64", "7", "9", "(:wat::rete::core::i64::= (:wat::rete::core::Vector/first (:wat::rete::core::Vector {f}) :undefined 0) 7)", ""),
        ":wat::rete::core::List" => (":wat::core::i64", "7", "9", "(:wat::rete::core::i64::= (:wat::rete::core::List/first (:wat::rete::core::List {f}) :undefined 0) 7)", ""),
        ":wat::rete::core::foldl" => ("(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 1 2)", "(:wat::core::PersistentVector 9)", "(:wat::rete::core::i64::= (:wat::rete::core::foldl (:wat::rete::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ acc x :undefined 0)) 0 {f}) 3)", ""),
        ":wat::rete::core::reduce" => ("(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 1 2)", "(:wat::core::PersistentVector 9)", "(:wat::rete::core::i64::= (:wat::rete::core::reduce (:wat::rete::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ acc x :undefined 0)) 0 {f}) 3)", ""),
        ":wat::rete::core::PersistentMap" => (":wat::core::String", "\"a\"", "\"z\"", "(:wat::rete::core::PersistentMap/contains-key? (:wat::rete::core::PersistentMap {f} 1) \"a\")", ""),
        ":wat::rete::core::mapv" => ("(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 1 2)", "(:wat::core::PersistentVector 9)", "(:wat::rete::core::i64::= (:wat::rete::core::Vector/first (:wat::rete::core::mapv (:wat::rete::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::* x 10 :undefined 0)) {f}) :undefined 0) 10)", ""),
        ":wat::rete::core::filterv" => ("(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 9)", "(:wat::core::PersistentVector 1)", "(:wat::rete::core::i64::= (:wat::rete::core::Vector/first (:wat::rete::core::filterv (:wat::rete::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::rete::core::i64::> x 5)) {f}) :undefined 0) 9)", ""),
        ":wat::rete::core::Tuple" => (":wat::core::i64", "7", "9", "(:wat::rete::core::i64::= (:wat::rete::core::Tuple/first (:wat::rete::core::Tuple {f} 99)) 7)", ""),
        ":wat::rete::core::Tuple/first" => (":wat::core::i64", "7", "9", "(:wat::rete::core::i64::= (:wat::rete::core::Tuple/first (:wat::rete::core::Tuple {f} 99)) 7)", ""),
        ":wat::rete::core::Tuple/second" => (":wat::core::i64", "7", "9", "(:wat::rete::core::i64::= (:wat::rete::core::Tuple/second (:wat::rete::core::Tuple 99 {f})) 7)", ""),
        ":wat::rete::core::Tuple/third" => (":wat::core::i64", "7", "9", "(:wat::rete::core::i64::= (:wat::rete::core::Tuple/third (:wat::rete::core::Tuple 99 99 {f})) 7)", ""),
        ":wat::rete::core::keyword/to-string" => (":wat::core::keyword", ":alpha", ":beta", "(:wat::rete::core::string::= (:wat::rete::core::keyword/to-string {f}) \"alpha\")", ""),
        ":wat::rete::core::keyword/from-string" => (":wat::core::String", "\"alpha\"", "\"beta\"", "(:wat::rete::core::string::= (:wat::rete::core::keyword/to-string (:wat::rete::core::keyword/from-string {f} :undefined :none)) \"alpha\")", ""),

        // ── THE FOUR HOLON ROWS. Excluded until 2026-08-28 on the stated ground that "a holon has
        // no literal spelling, so the second operand cannot be written as a constant the way every
        // scalar row's can". THAT WAS FALSE. `#holon <form>` is the literal — a holon holds the
        // same data EDN does, so it spells the same way — and the exclusion had measured a MISSING
        // LOWERING ARM (`cannot lower head :wat::holon::literal`) and written it down as an
        // impossibility. Builder: "holon is just another holder for data like edn is." The arm now
        // folds `#holon` to a constant at lower time, and these four need NOTHING the scalar rows
        // do not: one field, one literal rhs.
        //
        // THRESHOLDS ARE MEASURED, NOT GUESSED (`probe-holon-rete-cell-values.wat`): cosine is 1.0
        // for the hit and -0.018 for the miss; dot is 4333.0 and -81.0. `coincident?`/`presence?`
        // answer bool directly. A guessed threshold is a cell that can pass for the wrong reason.
        ":wat::rete::holon::coincident?" => (":wat::holon::HolonAST", "#holon [1 2 3]", "#holon [7 8 9]", "(:wat::rete::holon::coincident? {f} #holon [1 2 3])", ""),
        ":wat::rete::holon::presence?" => (":wat::holon::HolonAST", "#holon [1 2 3]", "#holon [7 8 9]", "(:wat::rete::holon::presence? {f} #holon [1 2 3])", ""),
        ":wat::rete::holon::cosine" => (":wat::holon::HolonAST", "#holon [1 2 3]", "#holon [7 8 9]", "(:wat::rete::core::f64::> (:wat::rete::holon::cosine {f} #holon [1 2 3] :undefined 0.0) 0.9)", ""),
        ":wat::rete::holon::dot" => (":wat::holon::HolonAST", "#holon [1 2 3]", "#holon [7 8 9]", "(:wat::rete::core::f64::> (:wat::rete::holon::dot {f} #holon [1 2 3] :undefined 0.0) 1000.0)", ""),
        _ => return None,
    };
    Some(t)
}

fn operands_for(rete_name: &'static str) -> Option<Cell> {
    if let Some((field_ty, hit, miss, expr, extra)) = special_for(rete_name) {
        return Some(Cell {
            op: rete_name,
            arity: 0,
            expr_is_verbatim: true,
            field_ty,
            hit,
            miss,
            extra,
            expr: expr.to_string(),
        });
    }
    let (arity, field_ty, hit, miss, rhs, wrap): (usize, &str, &str, &str, &str, Option<(&str, &str)>) = match rete_name {
        // i64 — the baseline. Note `<` needs its hit/miss SWAPPED relative to `>` against the
        // same literal, which is the whole argument for a per-row table.
        ":wat::rete::core::i64::>" => (2, ":wat::core::i64", "42", "3", "10", None),
        ":wat::rete::core::i64::<" => (2, ":wat::core::i64", "3", "42", "10", None),
        ":wat::rete::core::i64::>=" => (2, ":wat::core::i64", "10", "3", "10", None),
        ":wat::rete::core::i64::<=" => (2, ":wat::core::i64", "10", "42", "10", None),
        ":wat::rete::core::i64::=" => (2, ":wat::core::i64", "10", "3", "10", None),
        ":wat::rete::core::i64::not=" => (2, ":wat::core::i64", "3", "10", "10", None),

        // f64 — `>=`/`<=` pin the BOUNDARY (hit == rhs), so an implementation that dropped the
        // `=` half would go red here rather than passing on the strict half alone.
        ":wat::rete::core::f64::>" => (2, ":wat::core::f64", "42.0", "3.0", "10.0", None),
        ":wat::rete::core::f64::<" => (2, ":wat::core::f64", "3.0", "42.0", "10.0", None),
        ":wat::rete::core::f64::>=" => (2, ":wat::core::f64", "10.0", "3.0", "10.0", None),
        ":wat::rete::core::f64::<=" => (2, ":wat::core::f64", "10.0", "42.0", "10.0", None),
        ":wat::rete::core::f64::=" => (2, ":wat::core::f64", "10.0", "3.0", "10.0", None),
        ":wat::rete::core::f64::not=" => (2, ":wat::core::f64", "3.0", "10.0", "10.0", None),

        // String — the three predicates use a needle that is a strict INFIX/PREFIX/SUFFIX of the
        // hit and absent from the miss, so each one tests its own half rather than plain equality.
        ":wat::rete::core::String/starts-with?" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"al\"", None),
        ":wat::rete::core::String/ends-with?" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"ha\"", None),
        ":wat::rete::core::String/contains?" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"lph\"", None),
        ":wat::rete::core::String/empty?" => (1, ":wat::core::String", "\"\"", "\"x\"", "", None),
        ":wat::rete::core::string::=" => (2, ":wat::core::String", "\"alpha\"", "\"beta\"", "\"alpha\"", None),
        ":wat::rete::core::string::not=" => (2, ":wat::core::String", "\"beta\"", "\"alpha\"", "\"alpha\"", None),

        // bool
        ":wat::rete::core::not" => (1, ":wat::core::bool", "false", "true", "", None),
        ":wat::rete::core::bool::=" => (2, ":wat::core::bool", "true", "false", "true", None),
        ":wat::rete::core::bool::not=" => (2, ":wat::core::bool", "false", "true", "true", None),

        // keyword — `=` is the motivating asymmetry; `not=` is one of the three rows that appear
        // NOWHERE in the 1569-file corpus, so its cells are the first evidence it has ever had.
        ":wat::rete::core::keyword::=" => (2, ":wat::core::keyword", ":alpha", ":beta", ":alpha", None),
        ":wat::rete::core::keyword::not=" => (2, ":wat::core::keyword", ":beta", ":alpha", ":alpha", None),

        // Containers — a parametric field, so these also test that the template survives a
        // non-scalar declaration.
        ":wat::rete::core::PersistentVector/contains?" => (
            2,
            "(:wat::core::PersistentVector :- [:wat::core::i64])",
            "(:wat::core::PersistentVector 1 2)",
            "(:wat::core::PersistentVector 9)",
            "1",
            None,
        ),
        ":wat::rete::core::PersistentMap/contains-key?" => (
            2,
            "(:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])",
            "(:wat::core::PersistentMap \"a\" 1)",
            "(:wat::core::PersistentMap \"z\" 1)",
            "\"a\"",
            None,
        ),


        // ─── NON-`bool` ROWS — each WRAPPED in a comparator, because a value is not a condition.
        // The `Fallback` rows carry the mandatory `:undefined <value>` marker pair as their last
        // two operands, which is exactly how a partial core op BUYS totality here (`i64::/` is
        // partial in core and `total: true` in the table for this reason).

        // i64 arithmetic. `mod`/`rem` use expected 0 so the MISS lands on a nonzero remainder
        // rather than on a different quotient — otherwise a stubbed op returning 0 would pass.
        ":wat::rete::core::i64::+" => (4, ":wat::core::i64", "10", "1", "2 :undefined 0", Some((":wat::rete::core::i64::=", "12"))),
        ":wat::rete::core::i64::-" => (4, ":wat::core::i64", "10", "1", "2 :undefined 0", Some((":wat::rete::core::i64::=", "8"))),
        ":wat::rete::core::i64::*" => (4, ":wat::core::i64", "10", "1", "2 :undefined 0", Some((":wat::rete::core::i64::=", "20"))),
        ":wat::rete::core::i64::/" => (4, ":wat::core::i64", "10", "1", "2 :undefined 0", Some((":wat::rete::core::i64::=", "5"))),
        ":wat::rete::core::i64::mod" => (4, ":wat::core::i64", "10", "1", "2 :undefined 0", Some((":wat::rete::core::i64::=", "0"))),
        ":wat::rete::core::i64::rem" => (4, ":wat::core::i64", "10", "1", "2 :undefined 0", Some((":wat::rete::core::i64::=", "0"))),
        ":wat::rete::core::i64::quot" => (4, ":wat::core::i64", "10", "1", "2 :undefined 0", Some((":wat::rete::core::i64::=", "5"))),

        // f64 arithmetic. `f64::-` is one of the three rows appearing NOWHERE in the corpus.
        ":wat::rete::core::f64::+" => (4, ":wat::core::f64", "10.0", "1.0", "2.0 :undefined 0.0", Some((":wat::rete::core::f64::=", "12.0"))),
        ":wat::rete::core::f64::-" => (4, ":wat::core::f64", "10.0", "1.0", "2.0 :undefined 0.0", Some((":wat::rete::core::f64::=", "8.0"))),
        ":wat::rete::core::f64::*" => (4, ":wat::core::f64", "10.0", "1.0", "2.0 :undefined 0.0", Some((":wat::rete::core::f64::=", "20.0"))),
        ":wat::rete::core::f64::/" => (4, ":wat::core::f64", "10.0", "1.0", "2.0 :undefined 0.0", Some((":wat::rete::core::f64::=", "5.0"))),

        // String / scalar conversions.
        ":wat::rete::core::String/concat" => (2, ":wat::core::String", "\"a\"", "\"b\"", "\"x\"", Some((":wat::rete::core::string::=", "\"ax\""))),
        ":wat::rete::core::string::length" => (1, ":wat::core::String", "\"abc\"", "\"z\"", "", Some((":wat::rete::core::i64::=", "3"))),
        ":wat::rete::core::string::trim" => (1, ":wat::core::String", "\" a \"", "\"b\"", "", Some((":wat::rete::core::string::=", "\"a\""))),
        ":wat::rete::core::string::to-lowercase" => (1, ":wat::core::String", "\"A\"", "\"b\"", "", Some((":wat::rete::core::string::=", "\"a\""))),
        ":wat::rete::core::string::subs" => (5, ":wat::core::String", "\"abcd\"", "\"zzzz\"", "0 2 :undefined \"\"", Some((":wat::rete::core::string::=", "\"ab\""))),
        ":wat::rete::core::i64::to-f64" => (1, ":wat::core::i64", "3", "9", "", Some((":wat::rete::core::f64::=", "3.0"))),
        ":wat::rete::core::i64::to-string" => (1, ":wat::core::i64", "3", "9", "", Some((":wat::rete::core::string::=", "\"3\""))),
        ":wat::rete::core::f64::to-string" => (1, ":wat::core::f64", "3.0", "9.0", "", Some((":wat::rete::core::string::=", "\"3\""))),
        ":wat::rete::core::bool::to-string" => (1, ":wat::core::bool", "true", "false", "", Some((":wat::rete::core::string::=", "\"true\""))),

        // Container accessors. `first`/`get` return the ELEMENT type, so the wrap is the
        // element's comparator — the row's `Var("T")` return resolved by the field declaration.
        ":wat::rete::core::PersistentVector/length" => (1, "(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 1 2)", "(:wat::core::PersistentVector 9)", "", Some((":wat::rete::core::i64::=", "2"))),
        ":wat::rete::core::PersistentVector/get" => (4, "(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 7)", "(:wat::core::PersistentVector 9)", "0 :undefined 0", Some((":wat::rete::core::i64::=", "7"))),
        ":wat::rete::core::Vector/get" => (4, "(:wat::core::Vector :- [:wat::core::i64])", "(:wat::core::Vector :wat::core::i64 7)", "(:wat::core::Vector :wat::core::i64 9)", "0 :undefined 0", Some((":wat::rete::core::i64::=", "7"))),
        ":wat::rete::core::List/get" => (4, "(:wat::core::List :- [:wat::core::i64])", "(:wat::core::List/of 7)", "(:wat::core::List/of 9)", "0 :undefined 0", Some((":wat::rete::core::i64::=", "7"))),
        ":wat::rete::core::PersistentVector/first" => (3, "(:wat::core::PersistentVector :- [:wat::core::i64])", "(:wat::core::PersistentVector 7)", "(:wat::core::PersistentVector 9)", ":undefined 0", Some((":wat::rete::core::i64::=", "7"))),
        ":wat::rete::core::Vector/first" => (3, "(:wat::core::Vector :- [:wat::core::i64])", "(:wat::core::Vector :wat::core::i64 7)", "(:wat::core::Vector :wat::core::i64 9)", ":undefined 0", Some((":wat::rete::core::i64::=", "7"))),
        ":wat::rete::core::List/first" => (3, "(:wat::core::List :- [:wat::core::i64])", "(:wat::core::List/of 7)", "(:wat::core::List/of 9)", ":undefined 0", Some((":wat::rete::core::i64::=", "7"))),

        _ => return None,
    };
    Some(Cell {
        op: rete_name,
        arity,
        expr_is_verbatim: false,
        field_ty,
        hit,
        miss,
        extra: "",
        expr: uniform_call(rete_name, arity, rhs, wrap),
    })
}

/// Rows that ARE generable and are nonetheless excluded, because the compiled `where` EXECUTOR
/// cannot run them — a DEFECT, deliberately not filed under `NOT_YET_GENERABLE`.
///
/// ⛔ **Calling this a tooling gap would be the mislabel this ledger exists to prevent.** These
/// cells synthesize fine; they raise
/// `#wat.runtime/MalformedForm "compiled apply cannot dispatch kind Unknown arity N"` at RUNTIME,
/// after passing admission, totality, arity and type — the same shape as
/// `PersistentMap/contains-key?`, which was found the same way and FIXED (see `expr_ir.rs`).
///
/// Verified structurally, not inferred from the message: `expr_ir.rs`'s `CoreKind` mapping carries
/// `PvNew`/`VecNew`/`ListNew` for the three sibling constructors and has NO arm for either of
/// these. Note that a missing arm is not on its own proof of a hole — `foldl` and the other HOFs
/// map to `Unknown` too and reach the compiled path by their own dedicated route (`expr_ir.rs`
/// line ~371). What makes these two a defect is the CONJUNCTION: no arm, no other route, and a
/// runtime raise when driven.
///
/// **The extirpation is a gate, not two arms.** `RETE_OPS` and the `CoreKind` mapping are two
/// lists that must agree and nothing checks that they do; `holon_rete_ops_have_opexec` checks it
/// for holon rows only, and its own doc used to instruct the reader not to widen it. That widening
/// is its own strike — see `RETE-OPEN-WORK` § 4.1.
const COMPILED_EXECUTOR_CANNOT_RUN: &[(&str, &str)] = &[
];

/// The rows deliberately NOT in the operand table, each with the reason a reader can argue with.
///
/// An exclusion is a claim that a cell cannot be written, which is exactly the kind of claim this
/// arc has been wrong about twice (see the breadcrumb: "do not trust a grep that found nothing").
/// So each one names what would refute it.
const NOT_YET_GENERABLE: &[(&str, &str)] = &[];

/// The sweep body, run as one SHARD of the row list.
///
/// Sharded because 55 rows x 2 positions is 110 full program loads — ~30s serially, past the
/// runner's deliberate 30s kill (`.config/nextest.toml`, and that deadline exists to turn a
/// deadlock into a clean failure rather than a wedged run). Weakening the deadline for one test
/// would blunt it for every test; nextest already runs tests in parallel PROCESSES, so splitting
/// the work is free speed and leaves the gate exactly as strong.
///
/// ⛔ The partition is by INDEX, never by family. A hand-picked family split silently stops
/// covering a row whose family nobody added, which is the same "a list nobody re-reads" defect
/// this ledger exists to catch.
///
/// These 55 rows are the ones `RETE_OPS` gives enough to build a call from: they carry `params`
/// and a `ret`. Bool-returning rows are already constraint-shaped; the rest return a VALUE and are
/// wrapped in a comparator (see `Cell::wrap`). The remaining 19 — `Form` and `Redispatch` — carry
/// NO params and no scheme at all, so nothing here can synthesize them; they are a separate strike
/// and are deliberately not guessed at, since an un-calibrated shape is how a template
/// manufactures a column of false findings.
///
/// **What this gate makes impossible:** minting a rete row that no user can reach. `RETE_OPS`
/// gates purity, totality, arity and type; none of them asks whether the op can be CALLED. A new
/// Bool-returning `Alias` row with no operand entry fails here by name.
///
/// The full matrix prints on every run, pass or fail. A ledger whose output is only visible when
/// it breaks is a ledger nobody reads.
fn sweep_shard(shard: usize, of: usize) {
    let all: Vec<&ReteOp> = RETE_OPS.iter().collect();
    // NON-VACUITY on the POPULATION, checked before sharding — a shard of an empty set is empty
    // and would pass silently.
    assert!(
        all.len() >= 74,
        "RETE_OPS looks empty or renamed ({} rows) — this sweep would pass vacuously",
        all.len()
    );
    // Partition by INDEX, not by hand-picked family: every row lands in exactly one shard, so
    // sharding cannot create a row that no shard covers. A family split could.
    let rows: Vec<&ReteOp> =
        all.into_iter().enumerate().filter(|(i, _)| i % of == shard).map(|(_, r)| r).collect();

    let mut unclassified: Vec<&str> = Vec::new();
    let mut defects: Vec<String> = Vec::new();
    let mut silent: Vec<String> = Vec::new();
    let mut matrix: Vec<String> = Vec::new();

    for row in &rows {
        let Some(cell) = operands_for(row.rete_name) else {
            if let Some((_, why)) = COMPILED_EXECUTOR_CANNOT_RUN.iter().find(|(n, _)| *n == row.rete_name)
            {
                matrix.push(format!("{:<46} {:>17}  {}", row.rete_name, "NO-COMPILED-ARM", why));
            } else if let Some((_, why)) =
                NOT_YET_GENERABLE.iter().find(|(n, _)| *n == row.rete_name)
            {
                matrix.push(format!("{:<46} {:>17}  {}", row.rete_name, "NOT-GENERABLE", why));
            } else {
                unclassified.push(row.rete_name);
            }
            continue;
        };
        // The row's own arity is the authority; a table entry that disagrees is a table bug, and
        // silently trusting either one would let the two drift. A VERBATIM cell states its whole
        // expression, so its arity drives nothing and there is nothing to cross-check.
        if !cell.expr_is_verbatim {
            assert_eq!(
                cell.arity,
                row.params.len(),
                "operand table says arity {} for {} but the row declares {} params",
                cell.arity,
                row.rete_name,
                row.params.len()
            );
        }

        // Drive BOTH positions before judging either. `MatchedNothing` is ambiguous alone — bad
        // operands look identical to a broken position — and the other position is the control
        // that separates them: if the same cell discriminates somewhere, the operands are good.
        let sites = [CallSite::InlineConstraint, CallSite::WhereFence];
        let raw: Vec<(CallSite, String, Verdict)> = sites
            .iter()
            .map(|&site| {
                let src = synth(&cell, site);
                let v = drive(&src, cell.op);
                (site, src, v)
            })
            .collect();
        let any_fires = raw.iter().any(|(_, _, v)| matches!(v, Verdict::Fires));

        let mut verdicts: Vec<String> = Vec::new();
        for (site, src, verdict) in &raw {
            let label = site.label();
            // ADJUDICATION: a cell that matched nothing HERE, while the same cell discriminates
            // THERE, is rete accepting a clause it cannot satisfy — the operands are proven good
            // by the position that works, so the ambiguity is resolved and it becomes a finding.
            let adjudicated = match verdict {
                Verdict::TemplateDefect(DefectKind::MatchedNothing, _) if any_fires => {
                    Verdict::MatchesNothing
                }
                // An UNATTRIBUTED refusal is also adjudicated by the sibling position, and the
                // reasoning is the same one step further. The expression text is IDENTICAL in both
                // positions — only `{f}` renders differently — so a cell that fires anywhere is
                // valid wat, and a refusal elsewhere is therefore about the POSITION, whatever the
                // diagnostic happened to name.
                //
                // This is not a loosening; it closes a real blind spot. Some positions fail BEFORE
                // reaching the op at all: inline, `(enum::= {f} :probe::E::A)` refuses with
                // "`:probe::In` has no field `:probe::E::A`", because in operand position a bare
                // keyword is a FIELD REFERENCE. No diagnostic will ever name the op there, so
                // name-matching alone can only call a genuine finding a template bug.
                Verdict::TemplateDefect(DefectKind::Unattributed, m) if any_fires => {
                    Verdict::Refused(m.clone())
                }
                other => other.clone(),
            };
            match adjudicated {
                Verdict::Fires => verdicts.push(format!("{label}=FIRES")),
                Verdict::Refused(_) => verdicts.push(format!("{label}=REFUSED")),
                // ⛔⛔ THE CLASS GATE — fix-list entry **F** made structural.
                //
                // A cell must FIRE or be REFUSED. Those are the only two honest answers: one
                // works, the other TEACHES. "Compiled, ran, matched nothing, said nothing" is the
                // third, and it is the defect class this ledger exists to make impossible — a
                // wrong answer no fuzzer can see (both engines agree on the empty result) and no
                // reading ward can find (every gate the form passes is correct about it).
                //
                // It is asserted for EVERY row in EVERY position rather than for the one operand
                // shape that was broken, because the next instance of this will not be a nested
                // call. Entry F was 39 rows wide and nobody noticed for the life of the engine.
                Verdict::MatchesNothing => {
                    verdicts.push(format!("{label}=MATCHES-NOTHING"));
                    silent.push(format!(
                        "  {} @ {label} — compiled, fired, matched NOTHING, and said nothing.\n\
                         ─── the program driven ───\n{src}",
                        cell.op
                    ));
                }
                Verdict::TemplateDefect(kind, detail) => {
                    verdicts.push(format!("{label}=DEFECT"));
                    defects.push(format!(
                        "  {} @ {label}: {kind:?} — {detail}\n─── the program driven ───\n{src}",
                        cell.op
                    ));
                }
            }
        }
        matrix.push(format!("{:<46} {}", row.rete_name, verdicts.join("  ")));
    }

    println!("\n─── RETE_OPS reachability, Alias + Fallback — shard {shard}/{of} ───");
    for line in &matrix {
        println!("{line}");
    }
    println!("─── {} rows in this shard ───\n", rows.len());

    assert!(
        unclassified.is_empty(),
        "these rete rows have NO reachability verdict — a row that no cell exercises is a row \
         nobody has shown a user can reach. Add an operand entry, or an argued exclusion in \
         NOT_YET_GENERABLE: {unclassified:#?}"
    );
    assert!(
        silent.is_empty(),
        "SILENT WRONG ANSWER — a user form was accepted, compiled, fired, and matched nothing with \
         no diagnostic. This is fix-list entry F's class and it must never return: make the form \
         WORK (lower it through the one expression core, as `Op::Eval` does) or make it REFUSE by \
         name. Never neither.\n{}",
        silent.join("\n\n")
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
                Verdict::MatchesNothing => "MATCHES-NOTHING",
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


/// ★★ THE INVENTORY GATE — every row is classified, checked WITHOUT driving anything.
///
/// Separated from the sweep deliberately: this is the property that makes minting an unreachable
/// row impossible, and it must not be able to fail for a slow or flaky reason. It runs in
/// milliseconds and answers one question — does every `Alias`/`Fallback` row have either operand
/// data or an argued exclusion?
#[test]
fn every_rete_ops_row_is_classified() {
    let rows: Vec<&ReteOp> = RETE_OPS.iter().collect();
    assert!(rows.len() >= 74, "population looks empty ({}) — vacuous", rows.len());

    let unclassified: Vec<&str> = rows
        .iter()
        .filter(|r| {
            operands_for(r.rete_name).is_none()
                && !NOT_YET_GENERABLE.iter().any(|(n, _)| *n == r.rete_name)
                && !COMPILED_EXECUTOR_CANNOT_RUN.iter().any(|(n, _)| *n == r.rete_name)
        })
        .map(|r| r.rete_name)
        .collect();
    assert!(
        unclassified.is_empty(),
        "these rete rows have NO reachability classification — a row nothing exercises is a row \
         nobody has shown a user can reach. Give it operand data, or an argued entry in \
         NOT_YET_GENERABLE (a tooling gap) or COMPILED_EXECUTOR_CANNOT_RUN (a defect): \
         {unclassified:#?}"
    );

    // The exclusion list may not name a row that does not exist, or it becomes a place stale
    // entries hide — the same rot the operand table is protected from by being driven.
    let ghosts: Vec<&str> = NOT_YET_GENERABLE
        .iter()
        .chain(COMPILED_EXECUTOR_CANNOT_RUN.iter())
        .map(|(n, _)| *n)
        .filter(|n| !RETE_OPS.iter().any(|o| o.rete_name == *n))
        .collect();
    assert!(ghosts.is_empty(), "an exclusion list names rows that no longer exist: {ghosts:#?}");
}

#[test]
fn reachability_shard_0_of_6() { sweep_shard(0, 6) }
#[test]
fn reachability_shard_1_of_6() { sweep_shard(1, 6) }
#[test]
fn reachability_shard_2_of_6() { sweep_shard(2, 6) }
#[test]
fn reachability_shard_3_of_6() { sweep_shard(3, 6) }
#[test]
fn reachability_shard_4_of_6() { sweep_shard(4, 6) }
#[test]
fn reachability_shard_5_of_6() { sweep_shard(5, 6) }

/// Report the raw row count for a cell, bypassing the `Fires`/discrimination judgement.
///
/// The ledger deliberately collapses "fired but selected n != 1" into a defect, because for a
/// VERDICT that is all that matters. When investigating WHY a cell mismatched, the number itself
/// is the evidence, so this returns it.
#[cfg(test)]
fn raw_count(src: &str) -> Result<i64, String> {
    let world = match startup_from_source(src, None, Arc::new(crate::load::InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => return Err(format!("{e:?}")),
    };
    let func = world.symbols().get(":probe::run").cloned().ok_or("no entry")?;
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, crate::rust_caller_span!())
    })) {
        Ok(Ok(Value::i64(n))) => Ok(n),
        Ok(Ok(v)) => Err(format!("{v:?}")),
        Ok(Err(e)) => Err(format!("{e:?}")),
        Err(_) => Err("panic".to_string()),
    }
}

/// ★★ AN INLINE CONSTRAINT THAT COMPUTES NOW COMPUTES — fix-list entry **F**, closed.
///
/// ⚠ **THIS TEST USED TO ASSERT THE OPPOSITE, AND THAT IS WHY IT IS HERE.** It was written on
/// 2026-08-28 to pin a live defect: an inline constraint whose operand was a nested call was
/// accepted at every gate, compiled, fired, and matched NOTHING — for every fact, at exit code 0,
/// with zero bytes on stderr. The engine answered "no rows" to a rule whose correct answer was
/// one row, and no diagnostic was emitted anywhere.
///
/// The mechanism was four conspiring bails, and the last one is the one that mattered:
/// `compile_operand_expr` had a three-case mini-lowering that stopped at literals, so a nested
/// operand returned `None`, and `compiled_cond.rs`'s
/// `match (lhs_e, rhs_e) { (Some,Some) => Cmp, _ => ops.push(Op::Fail) }` turned that into a
/// COMPILED, PERMANENT, SILENT never-match. `Op::Fail` is correct for an operand that genuinely
/// cannot resolve — an unbound `?var`, a field the class does not declare — and a nested call was
/// falling into that bucket only because the lowering could not build one.
///
/// **The fix was not a wall.** The first proposal was to refuse the form at compile time, and the
/// builder refused that: *"we made it such that every rete form can be compiled to a jump table...
/// why is this any exception?"* It is not one. `compiled_cond` already imported the one core's
/// `Expr`; what never landed with flip 3 was the LOWERING. Now a nested operand goes through
/// `expr_ir::lower_in_frame` into the same `Expr::Call`, the same opcode, the same `RETE_OPS`
/// table the `where` fence uses, materialised by `Op::Eval`.
#[test]
fn an_inline_constraint_that_computes_now_computes() {
    let cell = operands_for(":wat::rete::core::i64::+").expect("row must be in the table");

    assert_eq!(
        raw_count(&synth(&cell, CallSite::InlineConstraint)),
        Ok(1),
        "10+2=12 selects exactly the hit fact. This read 0 before the fix — the whole of entry F"
    );
    assert_eq!(
        raw_count(&synth(&cell, CallSite::WhereFence)),
        Ok(1),
        "and the fence, which always worked, still answers the same. The two positions agreeing on \
         identical input is the property; they disagreed for the entire life of the engine"
    );

    // ⛔ THE DISCRIMINATION CONTROL. Without it this passes against a change that makes the
    // constraint always TRUE — which would also stop it 'matching nothing' while being just as
    // wrong. Asking for a value the arithmetic does NOT produce must select zero rows.
    let never = synth(&cell, CallSite::InlineConstraint).replace(":undefined 0) 12", ":undefined 0) 99");
    assert_eq!(
        raw_count(&never),
        Ok(0),
        "expecting 99 from 10+2 must select NOTHING — otherwise the operand is not being compared, \
         only evaluated, and the cell would pass while proving nothing"
    );
}

/// ★★ A KEYWORD CONSTANT IS WRITABLE INLINE — via the constructor, not by changing the grammar.
///
/// **The gap this closes, measured.** `ast_literal_value` admits Int / Float / Bool / String
/// literals in operand position and NOT keyword — deliberately, because a bare keyword there is a
/// FIELD REFERENCE (`matcher.rs`). Every other scalar could express a constant inline; keyword
/// could not. And rete exposed NONE of core's seven keyword verbs — its only two keyword rows are
/// `=`/`not=`, which fall out of the generic equality family — so there was no constructor to
/// reach for either. Fewest rows of any type in the table, and the only one with no way to make a
/// value.
///
/// `keyword/from-string` closes it as a SIDE EFFECT of parity rather than as a special case. The
/// bare-keyword-is-a-field-reference rule is untouched and still documented; what changed is that
/// there is now a spelling for the other meaning.
///
/// The row is `Fallback` because `from-string` is genuinely partial — it raises on a leading ':'
/// and on an angle-type head — so the mandatory `:undefined` is what makes it `total: true`,
/// exactly as it does for `i64::/`.
#[test]
fn a_keyword_constant_is_writable_in_an_inline_constraint() {
    const SRC: &str = r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::keyword])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k)
     (:wat::rete::core::keyword::= :v
       (:wat::rete::core::keyword/from-string "alpha" :undefined :none)))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v :alpha)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v :beta)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;
    assert_eq!(
        raw_count(SRC),
        Ok(1),
        "the constructor must select :alpha and reject :beta. Before the keyword converter rows \
         this was UNWRITABLE — there was no spelling for a keyword constant in operand position"
    );

    // ⛔ THE DISCRIMINATION CONTROL. Without it this passes against a constructor that returns
    // something equal to everything, or a comparison that is always true — both of which would
    // also give a non-zero count while proving nothing.
    let never = SRC.replace(r#""alpha" :undefined"#, r#""zeta" :undefined"#);
    assert_ne!(SRC, never, "the rewrite must change the constant");
    assert_eq!(
        raw_count(&never),
        Ok(0),
        "constructing :zeta must match NEITHER fact — otherwise the comparison is not comparing"
    );
}

/// ★★ A FIELD REFERENCE INSIDE A VECTOR BINDS LIKE ANY OTHER OPERAND — the `let` half of F's class.
///
/// ⚠ **THIS TEST WOULD HAVE FAILED THE DAY FIX-LIST F WAS DECLARED CLOSED**, and finding that is
/// the whole reason it exists. F closed the case where the OPERAND was a nested call. It did not
/// close the case where the field reference sits inside a `[...]` — which is exactly where a `let`
/// binder lives — because `bind_field_refs` walked only `Keyword` and `List` and ended in
/// `other => Some(other.clone())`. `WatAST::Vector` fell into that catch-all and was cloned
/// UNTOUCHED, so `:v` never became a slot read, stayed a bare keyword, compared unequal to every
/// i64 forever, and the rule compiled, fired and matched NOTHING with no diagnostic.
///
/// **Both engines shared it**, exactly as they shared F: `matcher.rs`'s `rewrite_field_refs` is
/// the same two arms and the same catch-all. That is why 5612 fuzzed shapes were blind — the two
/// engines did not merely agree, they agreed on the same wrong answer. So this gate drives BOTH.
///
/// **The extirpation is the deleted wildcard, not this test.** `other => clone` meant two things
/// at once — "this node is a leaf, leave it alone" AND "I have no arm for this node" — and the
/// second meaning is what went silent. Every variant is now named in both functions, so a new
/// `WatAST` variant is a COMPILE ERROR rather than a silent never-match. That is rung three of the
/// ladder; this test is the worked example that proves the rung was climbed in the right place.
///
/// ⛔ **WHAT THIS GATE CANNOT REACH, stated rather than implied.** The new `Map | Set => None`
/// arms refuse, and nothing here drives them: `:wat::rete::lower` rejects both literals upstream
/// ("cannot lower", measured 2026-08-28), so no rete source can reach those arms. A gate built
/// from what the language admits cannot test what the language excludes. The arms are named
/// anyway, because the alternative is the catch-all that caused this.
#[test]
fn a_field_reference_inside_a_vector_binds_like_any_other_operand() {
    const SRC: &str = r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k)
     (:wat::rete::core::i64::= (:wat::rete::core::let [x :v] x) 10))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v 10)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;

    // `Ok(1)` is the assertion AND the discrimination in one: the defect read 0 (matched nothing),
    // and a constraint made vacuously TRUE would read 2 (matched the miss as well). Only a binder
    // that actually resolves to the field's value can select exactly the hit.
    assert_eq!(
        raw_count(SRC),
        Ok(1),
        "a `let` binding a FIELD must select exactly the hit. This read 0 for the life of the \
         engine — compiled, fired, silent"
    );

    // ⛔ THE ORACLE CARRIES THE IDENTICAL DEFECT, so a native-only gate would have watched this
    // class walk straight back in through the reference engine — which is the surface every
    // differential fuzzer scores against.
    let oracle = SRC.replace(":wat::rete::fire-rules ", ":wat::rete::fire-rules$oracle ");
    assert_ne!(SRC, oracle, "the rewrite must actually select the oracle");
    assert_eq!(
        raw_count(&oracle),
        Ok(1),
        "the $oracle must agree. Before this strike both engines answered 0 — agreement on a wrong \
         answer, which is what made 5612 fuzzed shapes blind to it"
    );

    // ⛔ THE POSITION CONTROL — the same expression in a `where` fence, which ALWAYS worked. Its
    // job is to keep the two positions pinned to each other: they disagreed here for the life of
    // the engine, and a regression that breaks the fence instead would otherwise read as green.
    const FENCE: &str = r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (?v <- :v))
   (:wat::rete::where
     (:wat::rete::core::i64::= (:wat::rete::core::let [x ?v] x) 10))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v 10)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;
    assert_eq!(
        raw_count(FENCE),
        Ok(1),
        "the fence answer is unchanged — the two positions must agree on identical input"
    );

    // ⛔ THE VACUITY CONTROL. A `let` that binds a CONSTANT never touches the rewriter at all, so
    // it passed even while the defect was live. Pinning it proves this gate is not measuring
    // "inline `let` works" — which was already true — but specifically that the FIELD reference
    // inside the binder vector now resolves.
    let constant_binder = SRC.replace("[x :v] x) 10", "[x 7] x) 7");
    assert_ne!(SRC, constant_binder, "the rewrite must change the binder");
    assert_eq!(
        raw_count(&constant_binder),
        Ok(2),
        "a constant binder matches BOTH facts and did so before the fix — if this reads 1, the \
         gate above is passing for a reason other than the one it claims"
    );
}

/// ★★ EVERY PROVABLY-BOOLEAN FORM IS ADMITTED INLINE — `cond`, `let`, `match`, `if`.
///
/// ⚠ **ALL FOUR WERE REFUSED, AND THE STATED REASON WAS WRONG.** The record said `cond`/`let`/
/// `match` are *"polymorphic in their body's type and the inline position has no type check that
/// could demand bool of them."* Both halves failed on the disk:
///
///   · **Polymorphic-in-the-body means the type is a FUNCTION of the body** — and the body is in
///     the AST. `head_is_boolean_rete_predicate` asked only the HEAD, read `row.ret` (a
///     PLACEHOLDER for `Form` rows) and stopped. Nothing was unknowable; nothing asked.
///   · **`cond` was not failing a type test at all.** `vocabulary.rs` documents that the macro
///     expander descends into `where` bodies ONLY, so an inline `cond` was never expanded to
///     nested `if` and reached the compiler as a head with no lowering arm — refused with
///     `"alpha 0 cond did not compile"`, a diagnostic that named nothing. The discriminating probe:
///     wrapping `cond` in a provably-bool head SATISFIED the type objection and was still refused,
///     which the type story cannot explain and the macro-boundary story predicts exactly.
///
/// **Why a shape-only pass can prove this.** Rete's vocabulary is closed and every row is
/// `pure · deterministic · total`. Totality means no supported expression can fail to have a value;
/// purity and determinism mean the value — and therefore the TYPE — is a function of the
/// subexpressions, all present in the AST. The builder's ruling, 2026-08-28: *"we very carefully
/// crafted rete's DSL to ensure every form a user can express can be compiled into our dag jump
/// tree… we just inappropriately denied access, poorly, to tooling we fully intended to support."*
///
/// ⛔ **THE REFUSAL HALF IS THE LOAD-BEARING HALF OF THIS GATE.** Admitting a form whose body is
/// NOT provably boolean would re-open fix-list F on the spot: the clause is required to evaluate
/// TRUE, so a non-bool body compares unequal forever and the rule silently never fires. The three
/// refusal rows below are therefore not decoration — without them this test passes against a change
/// that admits everything, which is strictly worse than the denial it replaced.
#[test]
fn every_provably_boolean_form_is_admitted_inline() {
    fn src(predicate: &str) -> String {
        format!(
            r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) {predicate})]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v 10)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#
        )
    }

    // ── ADMITTED. `Ok(1)` is assertion and discrimination in one: the defect read `Err` (refused),
    // and a form admitted but not actually applied would read 2 (the miss fact matching too).
    for (name, predicate) in [
        (
            "cond",
            "(:wat::rete::core::cond ((:wat::rete::core::i64::> :v 5) true) (:else false))",
        ),
        (
            "let",
            "(:wat::rete::core::let [x :v] (:wat::rete::core::i64::> x 5))",
        ),
        (
            "match",
            "(:wat::rete::core::match (:wat::rete::core::i64::> :v 5) (true true) (false false))",
        ),
        (
            "if",
            "(:wat::rete::core::if (:wat::rete::core::i64::> :v 5) true false)",
        ),
        // Nested: a `let` whose body is an `if` — the recursion, not just the top form.
        (
            "let-of-if",
            "(:wat::rete::core::let [x :v] \
             (:wat::rete::core::if (:wat::rete::core::i64::> x 5) true false))",
        ),
    ] {
        let s = src(predicate);
        assert_eq!(
            raw_count(&s),
            Ok(1),
            "`{name}` is provably boolean and must fire inline, selecting exactly the hit"
        );
        // ⛔ THE ORACLE, every time. `$native` and `$oracle` agreeing on a wrong answer is this
        // arc's repeat failure, so a native-only assertion is not evidence about the engine.
        let oracle = s.replace(":wat::rete::fire-rules ", ":wat::rete::fire-rules$oracle ");
        assert_ne!(s, oracle, "the rewrite must select the oracle");
        assert_eq!(raw_count(&oracle), Ok(1), "`{name}`: the $oracle must agree");
    }

    // ── ⛔ REFUSED, and this is the half that keeps fix-list F closed. Each body below is a
    // legitimate rete expression whose type is NOT boolean; admitting any of them would compile a
    // constraint that can never be true and match nothing, silently, forever.
    for (name, predicate) in [
        // A `let` whose body is a bare binder — an i64, not a bool.
        ("let-body-is-a-value", "(:wat::rete::core::let [x :v] x)"),
        // A `let` whose body is arithmetic — provably i64, provably NOT bool.
        (
            "let-body-is-arithmetic",
            "(:wat::rete::core::let [x :v] (:wat::rete::core::i64::+ x 1 :undefined 0))",
        ),
        // An `if` whose branches disagree. Neither the type nor the form is provable, and this is
        // exactly the case a head-only test could never have seen.
        (
            "if-branches-disagree",
            "(:wat::rete::core::if (:wat::rete::core::i64::> :v 5) true 1)",
        ),
    ] {
        assert!(
            raw_count(&src(predicate)).is_err(),
            "`{name}` is NOT provably boolean and must be REFUSED — admitting it compiles a \
             constraint that is never true and matches nothing, silently, which is fix-list F"
        );
    }
}

/// ★★ A KEYWORD OPERAND IS A FIELD REF IF IT NAMES A FIELD, ELSE A CONSTANT — the ONE RULE.
///
/// ⚠ **THE ENGINE WAS ALREADY DECIDING THIS CORRECTLY ONE LEVEL DOWN.** `(keyword::= :v :alpha)`
/// and `(enum::= :v :probe::E::A)` were refused for the life of the engine — "`:probe::In` has no
/// field `:alpha`" — while the IDENTICAL comparison, nested one level as an operand of another
/// call, fired and answered correctly. Measured 2026-08-28, both directions. The cause was two
/// answers to one question, ~120 lines apart in `compiled_cond.rs`: `bind_field_refs` ran
/// `field_names.position(...)` and fell through to a keyword literal; `compile_operand_expr` ran
/// the same lookup and returned `Unresolvable`.
///
/// **There was no ambiguity to resolve, which is the part the record got wrong.** It called this a
/// syntactic ambiguity. `:probe::E::A` carries `::` and a field name is a bare identifier
/// (`available fields: [k, v]`), so an enum variant could never have been a field reference at
/// all. And a bare `:alpha` is ambiguous ONLY when the class actually declares a field `alpha` —
/// which the rule resolves in the field's favour, exactly as before.
///
/// **This can only ADMIT programs, never change one.** A keyword operand naming no declared field
/// was a hard freeze error, so no program that compiles today contains one. The `field_wins` row
/// below is what proves the other half of that claim.
#[test]
fn a_keyword_operand_is_a_field_ref_or_a_constant_by_one_rule() {
    const KW: &str = r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::keyword])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (:wat::rete::core::keyword::= :v :alpha))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v :alpha)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v :beta)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;

    const EN: &str = r#"(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :probe::E])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (:wat::rete::core::enum::= :v :probe::E::A))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v :probe::E::A)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v :probe::E::B)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;

    for (name, src, other) in [("keyword", KW, ":beta"), ("enum", EN, ":probe::E::B")] {
        assert_eq!(
            raw_count(src),
            Ok(1),
            "`{name}::=` must be writable DIRECTLY inline, selecting exactly the hit. This was \
             refused as an unknown field while the same comparison fired when nested one deeper"
        );
        // ⛔ THE ORACLE. Its `resolve_operand` read the fact field and returned `None` on a miss,
        // which `eval_clause` maps to "no match" — so before this strike native answered 1 and the
        // oracle answered 0. Two engines quietly DISAGREEING is the same instrument failure as the
        // two of them quietly agreeing; only driving both catches either.
        let oracle = src.replace(":wat::rete::fire-rules ", ":wat::rete::fire-rules$oracle ");
        assert_ne!(src, oracle, "the rewrite must select the oracle");
        assert_eq!(raw_count(&oracle), Ok(1), "`{name}`: the $oracle must agree");

        // ⛔ DISCRIMINATION. A constant matching NEITHER fact must select nothing — otherwise the
        // operand is being evaluated but not compared, and the rows above prove nothing.
        let never = src.replacen(&format!("::= :v {other}"), "::= :v :zeta", 1);
        if never != *src {
            assert_eq!(
                raw_count(&never),
                Ok(0),
                "`{name}`: a constant equal to no fact must select NOTHING"
            );
        }
    }

    // ⛔⛔ THE FIELD REFERENCE STILL WINS — the backward-compatibility proof, and the load-bearing
    // row of this test. `:alpha` here IS a declared field, so it must be read as a FIELD, never as
    // the constant `:alpha`. If the rule had been "keyword is a constant", the hit fact would
    // compare `:x` against the constant `:alpha`, match nothing, and this would read 0.
    const FIELD_WINS: &str = r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::keyword  alpha <- :wat::core::keyword])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (:wat::rete::core::keyword::= :v :alpha))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v :x :alpha :x)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v :x :alpha :y)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;
    assert_eq!(
        raw_count(FIELD_WINS),
        Ok(1),
        "a keyword that NAMES A DECLARED FIELD is still a field reference — the hit has v == alpha \
         and the miss does not. This is what makes the rule purely additive: every program that \
         compiles today keeps its exact meaning"
    );
}

/// ★★ A MISTYPED FIELD STILL SAYS "no field", and says it ONCE.
///
/// The companion to the rule above, and the reason that rule is safe to state so broadly. Reading
/// a non-field keyword as a constant could have turned the single most common real mistake — a
/// typo'd field name — into a silent never-match. It does not, and the diagnostic did not degrade:
///
///   · rete has keyword-valued and enum-valued constants ONLY. At `i64::>` there is no constant a
///     keyword could be, so "you meant a field" is both true and the actionable thing to say.
///   · and it is said ONCE. A first cut reported the located `UnknownField` AND a type mismatch
///     advising *"use the rete comparator for `keyword`"* — which teaches the WRONG fix for a
///     typo. R29 `RVINA ERVDIT`: two ruins pointing opposite ways teach worse than one.
/// ★ THE HASH-DESTRUCTURE MATCH ARM — `{var :field …}` — IN BOTH POSITIONS.
///
/// Refused until 2026-08-28 as *"match map-destructure is not lowered in v1"*, which is a STATUS
/// and not a reason. Those two lines were the LAST `v1` refusal left in the rete expression core.
/// Core supports the form and drives `:md::Point{40,2}` -> 42 through it.
///
/// The design question was whether this arm is genuinely different from its settled sibling
/// `(:ns::Type/field ?x)`, which compiles its field index because class AND field are both in the
/// accessor head. It is not. Core must dispatch on the receiver at runtime because nothing
/// declares it; a rete `?p` gets its class from the fact pattern's declared field type, so **rete
/// has MORE static information here, not less.** The refusal had inherited core's
/// runtime-polymorphism problem into a place that does not have it.
///
/// ⛔ THE THIRD ROW IS THE LOAD-BEARING ONE. My first cut returned "arm does not match" for a
/// field the class does not declare. Core RAISES `UnknownField` there — verified, and it raises
/// even with a catch-all arm after it. Silently not-matching would have meant the same expression
/// answering differently in the two engines, AND would have turned a typo into a constraint that
/// compiles, fires and matches nothing: fix-list F's class, minted fresh. It raises now, and the
/// message carries the available fields because the ruin must teach.
#[test]
fn a_match_hash_destructure_binds_fields_in_both_positions() {
    fn program(condition: &str) -> String {
        format!(
            r#"(:wat::core::defrecord :probe::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :probe::In  [k <- :wat::core::String  p <- :probe::Point])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [{condition}]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :p (:probe::Point :x 40 :y 2))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :p (:probe::Point :x 1 :y 1))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#
        )
    }
    const SUM: &str = "(:wat::rete::core::match {SUBJ} ({vx :x  vy :y} \
                       (:wat::rete::core::i64::+ vx vy :undefined 0)))";

    // INLINE — `{f}` renders as the bare field keyword; the arm binds from the fact's own field.
    let inline = program(&format!(
        "(:probe::In (?k <- :k) (:wat::rete::core::i64::= {} 42))",
        SUM.replace("{SUBJ}", ":p")
    ));
    // FENCE — the control. This position was never the problem, so an inline-only failure is
    // positional rather than the form being broken.
    let fence = program(&format!(
        "(:probe::In (?k <- :k) (?p <- :p))\n   (:wat::rete::where (:wat::rete::core::i64::= {} 42))",
        SUM.replace("{SUBJ}", "?p")
    ));
    assert_eq!(
        raw_count(&inline),
        Ok(1),
        "a hash-destructure arm must bind and select exactly the hit (40+2=42; the miss is 1+1)"
    );
    assert_eq!(raw_count(&fence), Ok(1), "and identically in a `where` fence");

    // ⛔ AN UNDECLARED FIELD RAISES — it does NOT quietly fail to match.
    let typo = program(&format!(
        "(:probe::In (?k <- :k) (?p <- :p))\n   (:wat::rete::where (:wat::rete::core::i64::= {} 42))",
        "(:wat::rete::core::match ?p ({vz :nope} vz))"
    ));
    let verdict = raw_count(&typo).expect_err("an undeclared field must not be silently non-matching");
    for needle in ["nope", "probe::Point", "does not declare"] {
        // rune:lint(loose-assert) — targeted presence of three independent facts in one long EDN
        // error face; an exact assert_eq! would pin a span and a whole rendered diagnostic, which
        // is not what this row is about.
        assert!(
            verdict.contains(needle),
            "the diagnostic must name the field, the class, and WHY — core's `UnknownField` does, \
             and a rete row that answered differently would be the divergence this gate exists \
             for. missing {needle:?} in: {verdict}"
        );
    }

    // `{:keys […]}` is refused BY NAME rather than falling through to a generic "unsupported
    // pattern", so the diagnostic teaches the spelling that works. Core refuses it too.
    let keys = program(&format!(
        "(:probe::In (?k <- :k) (?p <- :p))\n   (:wat::rete::where (:wat::rete::core::i64::= {} 42))",
        "(:wat::rete::core::match ?p ({:keys [x y]} 1))"
    ));
    let kv = raw_count(&keys).expect_err("keys-destructure is not a match pattern");
    // rune:lint(loose-assert) — targeted presence in a long EDN error face, same reason as above.
    assert!(
        kv.contains("must be a bare"),
        "refusing `{{:keys …}}` must name the supported form; got: {kv}"
    );
}

/// A ROW THAT DECLARES `bool` IS BELIEVED INLINE, WHATEVER ITS CLASS — and one that declares
/// nothing is still refused. Both halves, or this proves nothing.
///
/// `:wat::rete::holon::coincident?` genuinely returns bool and is `OpClass::Redispatch`, because
/// its PARAMS keep core's `HolonAST | Vector` polymorphism and cannot be a rank-1 scheme. Its
/// return was collateral damage: `expr_is_provably_boolean` believed `ret` only for
/// `Alias`/`Fallback`, since on the other two classes `ret: ParamType::Bool` was a PLACEHOLDER
/// meaning "no scheme". One value, two facts — the fifth instance of that pattern in this arc.
///
/// So the op worked in a `where` fence and was REFUSED as an inline constraint —
/// "malformed rete clause … not a recognized :when shape" — for a full day, invisibly, because
/// nothing drove the inline position of a holon row.
///
/// ⛔ THE SECOND HALF IS NOT DECORATION. Widening the old guard to admit `Redispatch` "fixes"
/// `coincident?` and simultaneously admits `Tuple/first` — an `i64` whose row ALSO said
/// `ret: Bool` — as an inline boolean constraint that compiles, fires and SILENTLY MATCHES
/// NOTHING. That is fix-list F's class reopened. It was driven before it was proposed, and this
/// row is what keeps it dead: the fix must let `coincident?` through WITHOUT letting `Tuple/first`
/// through, which only a real `Ret::Is(Bool)` / `Ret::NoScheme` distinction can do.
#[test]
fn a_row_that_declares_bool_is_believed_inline_whatever_its_class() {
    const HOLON_DECLS: &str = r#"(:wat::core::defn :probe::alpha [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:wat::core::Vector :wat::core::i64 1 2 3)))
(:wat::core::defn :probe::beta [] -> :wat::holon::HolonAST
  (:wat::holon::to-holon (:wat::core::Vector :wat::core::i64 7 8 9)))

(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::holon::HolonAST  w <- :wat::holon::HolonAST])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])
"#;
    // Two HolonAST fields, not a literal: the ledger's own exclusion says a holon "has no literal
    // spelling", which is true and beside the point — one field cannot discriminate, because
    // `coincident?(h, h)` is true for every `h`. The hit fact matches; the miss fact does not.
    const HOLON_TAIL: &str = r#"
(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v (:probe::alpha) :w (:probe::alpha))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v (:probe::beta)  :w (:probe::alpha))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;
    let inline = format!(
        "{HOLON_DECLS}\n(:wat::rete::defrule :probe::rule\n  :when\n  \
         [(:probe::In (?k <- :k) (:wat::rete::holon::coincident? :v :w))]\n  :then\n  \
         [(:probe::Out :k ?k)])\n{HOLON_TAIL}"
    );
    let fence = format!(
        "{HOLON_DECLS}\n(:wat::rete::defrule :probe::rule\n  :when\n  \
         [(:probe::In (?k <- :k) (?v <- :v) (?w <- :w))\n   \
         (:wat::rete::where (:wat::rete::holon::coincident? ?v ?w))]\n  :then\n  \
         [(:probe::Out :k ?k)])\n{HOLON_TAIL}"
    );
    assert_eq!(
        raw_count(&inline),
        Ok(1),
        "`coincident?` declares `Ret::Is(Bool)` and must be admitted as an inline constraint. A \
         refusal here means a class test crept back into `expr_is_provably_boolean`"
    );
    assert_eq!(
        raw_count(&fence),
        Ok(1),
        "the `where` fence always accepted it — this arm is the CONTROL, and its job is to prove \
         an inline failure is positional rather than the op being broken"
    );

    // THE SOUNDNESS TWIN. `Tuple/first` returns the tuple's first element, an `i64`. Its row
    // declares `Ret::NoScheme`, so it must NOT be readable as a boolean predicate.
    const TUPLE: &str = r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (:wat::rete::core::Tuple/first (:wat::rete::core::Tuple :v 99)))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "a" :v 7)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;
    let verdict = raw_count(TUPLE);
    assert!(
        verdict.is_err(),
        "an `i64`-returning row must be REFUSED inline, not admitted. Admitting it yields a \
         constraint that compiles, fires and silently matches nothing — fix-list F's class. \
         Got: {verdict:?}"
    );
}

#[test]
fn a_mistyped_field_still_names_the_field_and_only_once() {
    const SRC: &str = r#"(:wat::core::defrecord :probe::In  [k <- :wat::core::String  celsius <- :wat::core::i64])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (:wat::rete::core::i64::> :celcius 5))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))
"#;
    let msg = raw_count(SRC).expect_err("a mistyped field must not compile");
    // rune:lint(loose-assert) — the refusal is a `ReteCheckErrors` batch embedding a Span path;
    // pin the TEACHING halves. The structured face is asserted exactly by validate.rs's own tests.
    assert!(
        msg.contains("has no field"),
        "the typo must still be reported as a missing FIELD, not as a type mismatch about a \
         keyword constant; got:\n{msg}"
    );
    // rune:lint(loose-assert) — same batch. This half is the count, and it is the whole point:
    // ONE error, so the author is not also told to switch comparator.
    assert!(
        msg.contains("1 rete rule validation error"),
        "exactly ONE error — a second, mismatch-flavoured one would teach the wrong fix; got:\n{msg}"
    );
}

# VIGILIA 2026-09-07 — the RETE subsystem, scoped

> **Builder's ruling, 2026-09-07:** *"we made a mistake last time... it was cast upon all of wat....
> we must scope this next vigilia to the rete subsystem.... the code must be the exemplar... the
> docs are the scaffolding, not the product."*

## The one rule this cast is built around

**SCOPE THE TARGET, NEVER THE WARD.** The previous cast covered the whole substrate and returned
178 findings, most of them main's. The correction is a narrower *target set* — not a shorter ward
roster. Every ward the kind-rules and triggers muster is cast; a ward that does not fire records
**why it did not**, with the measurement, so a reader can tell a measured zero from a skipped one.

⛔ **Hand-picking a roster is the dishonest shape `vigilia` names by name** — *"skipping spells whose
findings are inconvenient"* — and I proposed exactly that before the builder refused it. The docs
wards (`nesciens`, `cohaerere`, `consonare`) muster nowhere here **because no documentation is in
the target set**, which is a fact about the scope rather than a preference about the guard.

## The four targets, cast in sequence

Sequenced, not simultaneous: each cast is ~20 subagents, and every return is weighed against my own
read before the next fans out. Four at once is eighty workers and a board to triage instead of
strikes to run — the failure this scoping exists to prevent.

| # | target | files | lines | status |
|---|---|---|---|---|
| 1 | `src/rete/kernel/` + `wat/rete/oracle/` — the fire path and the spec it must mirror | 28 | 15,771 (measured) | ✅ **CAST COMPLETE** — 14/14 wards, 38 rows |
| 2 | `src/rete/**` minus `kernel/` + `wat/rete*.wat` — the compile side and its spec (⛔ WIDENED — see below) | 25 | 23,886 (measured) | CASTING |
| 3 | `wat-scripts/perf/grid/` — the load-bearing instrument and its corpus | 54 | ~8.7k | PENDING |
| 4 | `tests/rete/` + `src/rete/kernel/tests/` — the probe corpus | 264 | ~36k | PENDING |

## Muster, derived per target — with the triggers MEASURED

Cast 1, `src/rete/kernel/` + `wat/rete/oracle/`:

| ward | rule | verdict |
|---|---|---|
| intueri · solvere · conformare · purgare · struere · sequi · temperare | universal code | **muster** |
| exigere | universal, every kind | **muster** |
| cernere · probare · conferre | spec / language / DSL — `wat/rete/oracle/` is the spec | **muster** |
| perspicere | 2+ `<` in a type | **muster** — 14 files |
| excusare | runes / inline suppressions | **muster** — 18 files |
| secare | parallel primitives | **NO — 0 files.** A measured fact about the fire path |
| mora | a wait by chosen duration, or a timeout-0 snapshot | **NO — 0 files** |
| experiri | declares a callable surface | **NO here** — `RETE_OPS` is declared at `vocabulary.rs:307`, so it fires on target 2 |
| peragrare | a load-bearing instrument AND its corpus | **NO here** — fires on target 3 |
| partire | downstream of `solvere` reporting braiding at 2+ conflicting sites | evaluated after |
| circumspicere | always, last | **muster, last** |

Later targets get their own derivation block, written before that cast.

## Muster for target 2 — derived and MEASURED 2026-09-07, before the cast

⛔ **FIRST: THE TARGET AS ORIGINALLY SCOPED HAD A HOLE, AND IT IS FIXED HERE.** The table above
said target 2 was `src/rete/*.rs` — **top level only**, 15 files / 15,687 lines (both figures
confirmed exactly). But `src/rete/` has two subdirectories that are not `kernel/`:

    src/rete/expr_ir/   2 files, 2,689 lines
    src/rete/validate/  3 files, 3,147 lines

**Neither is in ANY of the four targets.** Target 1 was `kernel/` (minus tests), target 2 was the
top-level glob, target 3 is the grid, target 4 is the test corpus. **5,836 lines of rete code would
have been swept by nothing**, in a cast whose entire purpose is to make this subsystem an exemplar.

**And target 1's own return is what proves the hole was costly.** `conformare` reported — honestly,
and I credited it — that *"this target defines no error type of its own, and has no `From` conversion
impls to audit."* True. The rete error types are `ReteCheckErrorKind`, `ReteCheckError` and
`ReteCheckErrors`, and they live at **`src/rete/validate/error.rs:23,435,457`** — inside one of the
two uncovered directories. A ward answered "not here" correctly and pointed straight at ground the
scoping had left out. **Target 2 is therefore widened to `src/rete/**/*.rs` minus `kernel/`.**

⛔ **SECOND: TARGET 2 GETS ITS SPEC HALF, BY THE SAME PAIRING TARGET 1 USED.** Target 1 was the fire
path *plus the oracle it must mirror*. The compile side has its own spec half — `wat/rete.wat` and
`wat/rete/{compile,syntax,acc,factbag}.wat`, 5 files / 2,363 lines — and without it `conferre`,
`cernere` and `probare` have nothing to compare against. It is also live and load-bearing: the
`rule-produces` cure this session came from `compile.wat`'s own recipe. **7 `Mirrors <fn>` claims**
exist on the Rust side (`eval_test.rs:12`, `compiled_cond.rs:136,1399`, `validate/mod.rs:708,757`,
`vocabulary.rs:1291`) — each a spec claim, and that shape is exactly how this cast's only earlier
L1 was found.

**TARGET 2, FINAL: 25 files, 23,886 lines.**

| ward | rule | verdict — with the measurement |
|---|---|---|
| intueri · solvere · conformare · purgare · struere · sequi · temperare | universal code | **muster** |
| exigere | universal, every kind | **muster** |
| cernere · probare · conferre | spec / language / DSL | **muster** — the spec half is the 5 `wat/rete*.wat` files; 7 `Mirrors` claims to check |
| conformare | error types | **muster, and it FIRES here** — 3 error types at `validate/error.rs`; it measured **zero** on target 1 |
| perspicere | 2+ `<` in a type | **muster** — **16 of 20** `.rs` files; `reachability.rs` 27, `validate/mod.rs` 15, `expr_ir/eval.rs` 15. Only 2 files carry an existing `rune:perspicere` |
| excusare | runes / inline suppressions | **muster** — 33 `rune:` lines + 5 `#[allow(…)]`, across **15 of 20** files |
| experiri | declares a callable surface | ⭐ **MUSTER — and this is the target it was waiting for.** `RETE_OPS` is declared at `vocabulary.rs:307` with **108 rows**. It did NOT fire on target 1; that was recorded then and is discharged now |
| secare | parallel primitives | **NO — 0 files.** `grep -lE 'rayon\|par_iter\|thread::spawn\|std::sync::(Mutex\|RwLock)'` over all 20 → empty. A measured fact about the compile side |
| mora | a wait by chosen duration | **NO — 0 files.** `grep -lE 'sleep\|Duration::from\|timeout'` → empty |
| peragrare | a load-bearing instrument AND its corpus | **NO here** — fires on target 3 |
| partire | downstream of `solvere` reporting braiding at 2+ conflicting sites | evaluated after `solvere` returns |
| circumspicere | always, last | **muster, LAST** — target 1 proved why: cast last with the aggregate, it found the sharpest L1 on ground another ward had cleared |

⛔ **`experiri` IS SEQUENCED SEPARATELY, AND THIS IS ORDERING, NOT ROSTER-PICKING.** It is the one
ward whose evidence is an **event** rather than a source — it synthesizes and DRIVES each advertised
form. So (a) it cannot be briefed READ-ONLY like the other fourteen, and (b) driving wat needs a
build, and this repo's rule is one cargo build at a time. It is therefore cast in its own serialized
slot after the read-only wards return, not folded into a parallel wave. **Recording it here so a
later reader can tell a deliberate sequence from a quietly dropped ward** — the distinction this
whole README is built on.

## ⛔ How this directory is built against the last one's rot

Each rule below answers a specific failure this arc actually suffered:

1. **`FINDINGS.md` IS THE ONLY STATUS HOME.** The 09-05 cast kept status in `WORK-LIST.md` *and*
   `RETE-BOARD.md`; `WORK-LIST.md:10-13` said *"One row, one place"* and its neighbour, written four
   days later into the same directory, shipped its own status column. **Nine rows read OPEN while
   every one of them was already cured.** There is no second board here. `reports/` holds evidence,
   never status.
2. **EVERY ROW CARRIES ITS RE-DERIVATION** — the command, gate or grep that decides its status now.
   A row whose status cannot be re-derived is a claim, and claims rot silently. This is the direct
   cure for the nine.
3. **NO STATIC "RECOMMENDED ORDER".** The last board's said *"F1 first"* long after F1 was cured; a
   recovering instance following it would have struck a corpse. Priority is `vigilia`'s own rule —
   L1 before L2, most upstream ward first — applied at read time, against re-derived status.
4. **WARD RETURNS LAND IN `reports/` VERBATIM, BEFORE ANY SYNTHESIS.** The 2026-08-30 cast lost all
   nineteen of its returns because they existed only as subagent messages.
5. **A NON-FIRING TRIGGER IS RECORDED WITH ITS MEASUREMENT**, above. Absence of a ward is a result.

⚠ **`peragrare`'s census does NOT live here.** The spell requires the enumerator committed *beside
the corpus it counts* — so the grid's census goes in `wat-scripts/perf/grid/`, and target 3's block
records its path. A census nobody can re-run is a rumour with a grid.

⚠ **Nothing in `docs/` is gated.** `no_stale_path_in_doc`'s ROOTS are `src/rete`(.rs), `wat`,
`wat-tests`; `rete_citation_resolves` scans `src/rete` comments. Every `file:line` in this directory
is unchecked by construction — prefer naming a symbol or a gate over a bare line number, and expect
line citations here to rot.

---

# ⛔ HOW TO RESUME THIS CAST — read this FIRST if you are picking it up cold

**TARGET 1 IS COMPLETE — all fourteen wards cast, returned, and weighed against the disk.** The
casting procedure below is not recoverable from anything else on disk: it lived in the
orchestrator's context, and this section is the only copy. **Read it before casting target 2.**

## State — target 1 CLOSED 2026-09-07

**`src/rete/kernel/**` + `wat/rete/oracle/**` — 28 files, 15,771 lines (measured), 14 of 14 wards.**
**38 rows · 6 L1 · 22 L2 · 1 L3 · 9 in solvere's own vocabulary · 20 verified by the orchestrator.**
No code was changed by the cast — it was READ-ONLY by construction and every ward was briefed so.

| ward | verdict |
|---|---|
| intueri | 1 L2 |
| purgare | 2 L2 |
| solvere | 9 (2 structural ★) |
| struere | 6 L2 |
| conferre | **1 L1** + 2 L2 |
| sequi | **CLEAN** + 2 notes |
| temperare | 2 L2, 5/5 runes upheld |
| excusare | 65 weighed, 59 HOLD, **6 struck** |
| exigere | **CLEAN** + 1 wording row |
| conformare | 2 L2 (⚠ its report ends "CONVERGED" — see the convergence clause below) |
| cernere | 1 L2 — a phantom form in user-facing error text; ~130 oracle names all resolve |
| probare | **1 L1** + 1 L2 — a deferral resting on a citation that points at no call |
| perspicere | 3 L2; **all 10 runes CLEAR** (⚠ its report also ends "CONVERGED") |
| circumspicere | **1 L1** — a shipped ceiling contract sampled only at round/batch boundaries |

**⭐ WHAT THE FULL GUARD BOUGHT, since this is the evidence for casting it again on targets 2–4:**

- **Two wards came back CLEAN** (`sequi`, `exigere`) — results, not waste, because both said what
  they swept. `exigere` re-derived the zero TODO count independently rather than inheriting it.
- **Wards landed on the same site through different lenses** — `intueri`+`struere` (I1/T5),
  `cernere`+`conformare` (N1/F2, the same decode family: one says the error cannot name the user's
  line, the other that it cannot name a real form).
- **Three wards DISAGREED about one rune** (X3) — a decision for the builder, which no single cast
  could have produced.
- **The LAST ward found the sharpest L1** (W1), on ground `sequi` had already pronounced clean —
  correctly, on its own axis. That is the whole argument for `circumspicere` being cast last, and
  the whole argument against hand-picking a roster.
- **65 exemptions weighed, 59 upheld; all 10 `perspicere` runes upheld; the one
  `rune:circumspicere` upheld.** This subsystem's runes are overwhelmingly real — a fact only a
  cast ward can establish.

⛔ **NEXT: TARGET 2** (`src/rete/*.rs`, the compile side, 15 files). Derive its muster block with
MEASURED triggers before casting, as this one did. `experiri` fires there — `RETE_OPS` is declared
at `vocabulary.rs:307` — and it is the one ward that EXECUTES rather than reads, so scope it to a
target whose side effects are undoable.

**Targets 2, 3 and 4 have not been cast at all.** See the table above in this README. ⛔ Before
casting them, read the rewritten convergence clause below — the old wording split two of twelve
reports and must not be reused.

## The casting procedure — follow it exactly

1. **Fetch the ward's text from the datamancy MCP** (`fetch_spell` with the short name). Do NOT read
   a local copy: the signed manifest is the only trusted source, and a local copy is unverified,
   stale bytes.
2. **Spawn ONE subagent per ward** with the ward's full text **embedded verbatim** in the prompt.
   You cannot establish from here whether a worker can reach the MCP, so embedding is the default
   and the fallback both. One ward per worker — never a bundle.
3. **Every brief carries these, and they are what made the returns worth having:**
   - the exact target paths, and "read ONLY these; do not read the whole repo";
   - **READ-ONLY. No edits, no cargo runs.**
   - "Ground every finding in a `file:line` you actually read this session. No citation, no finding."
   - ⛔ **THE CONVERGENCE CLAUSE — REWRITTEN 2026-09-07 BECAUSE THE FIRST WORDING WAS AMBIGUOUS AND
     TWO OF TWELVE WARDS READ IT THE WRONG WAY.** It used to read: *"If it converges, say CONVERGED
     and say what you looked at."* `conformare` (`reports/conformare.md:59`) and `perspicere` both
     ended a report with **CONVERGED** while returning findings — `perspicere`'s very next sentence
     was *"The result is not 'clean': there are 13 genuine findings."* They read "converge" as
     **my sweep converged / I was exhaustive**, not **the target is clean**. Both readings are
     legitimate English and the wards answered honestly; the clause was the defect. Use this
     instead, verbatim:
     > "End with exactly one of two words. **CLEAN** — you found nothing to report; then say what
     > you swept, with the commands and counts, so a clean result is distinguishable from a cast
     > that never ran. **FINDINGS** — you are returning at least one, however small. Do not use the
     > word *converged*: it is ambiguous between 'the target is clean' and 'my sweep was thorough',
     > and reports have already split on it. If your sweep was exhaustive AND you found things, that
     > is **FINDINGS**, and you may say the sweep was exhaustive in your own words."
     (The value the old clause was reaching for is real and must be kept: `sequi` and `exigere` both
     came back clean *having said what they looked at*, which is what made them evidence rather than
     silence.)
   - **the PRIOR ART, by name** — settled work the ward must not re-report. Without it the count
     inflates with things already done. See each returned report for what was named.
   - any **scope correction** the ward needs (e.g. `conformare` was told this target owns no error
     types; `temperare` was told the oracle's naive replay is its CONTRACT, not waste).
4. **Write the return to `reports/<ward>.md` VERBATIM, before any synthesis.** The 2026-08-30 cast
   lost all nineteen of its returns because they lived only as subagent messages.
5. **Weigh each finding against your own read of the disk before rowing it.** Mark `✅ I VERIFIED`
   only for rows you re-read yourself; everything else is `⚠ ward-reported` and may not be cited as
   fact.
6. **Row into `FINDINGS.md` — the only status home — each with its re-derivation.**

## ⛔ Two rules the builder set, and why

- **SCOPE THE TARGET, NEVER THE WARD.** The orchestrator first proposed a hand-picked roster and the
  builder refused it: *"it feels like we should prove the others don't find anything instead of
  assuming they won't."* That was right, and it paid — `sequi` and `exigere` both CONVERGED (a
  result, not a waste), and `intueri`+`struere` landed on the same lines through different lenses,
  which neither alone would have shown. The docs wards muster nowhere here because no documentation
  is in the target set — a fact about the scope, not a preference about the guard.
- **A NON-FIRING TRIGGER IS A RESULT.** `secare` (0 parallel primitives) and `mora` (0 waits) do not
  muster on target 1 — measured, and recorded above as facts about the fire path.

## ⚠ Five counts of one population, none agreeing

The runes in target 1 have been counted by four wards and by the orchestrator: **41 / 54 / 56 / 57**
(and 58 raw `rune:` lines). No finding turns on it, and nobody is badly wrong — but **a count that
varies with who ran it needs its method stated beside it**, and no later claim should rest on one of
these figures unqualified.

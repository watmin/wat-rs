# FINDINGS — vigilia 2026-09-07, rete

> ⛔ **THIS FILE IS THE ONLY PLACE A ROW'S STATUS LIVES.** Edit in place; never append a closure
> below a row. `reports/` holds each ward's verbatim return as EVIDENCE — it carries no status, and
> a status read from a report rather than from this table is a status nobody maintains.
>
> The rule and its cost: `WORK-LIST.md:10-13` of the 2026-09-05 cast said *"One row, one place"*,
> and `RETE-BOARD.md` — same directory, four days later — shipped its own status column anyway.
> Both copies then rotted in lockstep: **nine rows read OPEN while every one was already cured.**

> ⭐ **EVERY ROW CARRIES A RE-DERIVATION** — the command, gate or grep that decides its status
> *now*. A row without one is a claim, and this arc has proved that claims rot silently while
> nobody re-checks them. If you cannot write the re-derivation, the finding is not yet understood
> well enough to row.

**Severity:** L1 = a correctness lie · L2 = a structural mumble · L3 = taste (noted, does not count
toward convergence). Passed through from each ward verbatim — this table never re-classifies.

**Priority is not written down.** Apply `vigilia`'s rule at read time — L1 before L2, most upstream
ward first — against status you have just re-derived. The last board's static order named a cured
row first for days.

## Rows

| id | ward | target | site | finding | sev | status | re-derivation |
|---|---|---|---|---|---|---|---|
| **I1** | intueri | 1 | `fire/pass/accumulate.rs:18` | `accumulate_pass`'s doc promises only *"dispatch the accumulate nodes"*; the body ALSO runs pass 3.20 — pre-dispatching `Test` parents that feed an accumulate, so `filter_pass` can skip them. A reader trusting the doc misses that a sibling pass's inputs are seeded here. | L2 | **OPEN** | `grep -q dispatch_where_tests src/rete/kernel/fire/pass/accumulate.rs && sed -n 18p … \| grep -qiv pre-dispatch` — closed when the doc names both responsibilities, or 3.20 becomes its own fn |
| **P1** | purgare | 1 | `wat/rete/oracle/fire.wat:231-254` | `retain-supported` is DEAD — superseded by `factbag::retain` during the factbag-one-owner strike (drawn this session) and never deleted. Zero call sites. | L2 | **OPEN** · ✅ I VERIFIED | `grep -rn 'retain-supported' wat/ wat-scripts/ wat-tests/ tests/ src/` → 3 hits, all in `fire.wat`, none a call |
| **P2** | purgare | 1 | `stratify.rs:444-471`, consumed `:1045` | `TerminationProof`'s two variants are constructed but the DISCRIMINANT is never read — the only consumer does `let _ = proof;`. | L2 | **OPEN** · ⚠ ward-reported | closed when something branches on the variants, or it collapses to a marker, or it carries a rune |
| **S1** | solvere | 1 | `stratify.rs:1-2` vs `:330-1067` | Module doc claims stratum-numbering only; ~69% of the file is a fixpoint-TERMINATION verifier — a different concern, unnamed in the header. | structural | **OPEN** · ⚠ ward-reported | closed by a split or an honest header |
| **S2** | solvere | 1 | `stratify.rs:532-544` | The two desugared constructor heads are hardcoded here AND maintained in `purity.rs`'s ratchet. Drift fails silently. | incidental | **OPEN** · ⚠ ward-reported | `grep -c 'kwargs-construct' …` — closed by one shared constant |
| **S3** | solvere | 1 | `fire/rules.rs:80-96` + `:522-530` | Forward→reverse children inversion hand-written twice in one file. | incidental | **OPEN** · ⚠ ward-reported | closed by one `build_rev_children` helper |
| **S4** | solvere | 1 | `fire/rules.rs:551-571` + `:401-416` | Query closure computed twice per non-monotonic requery — once for a boolean gate, once for the work. Defended on COST, not architecture. | incidental | **OPEN** · ⚠ ward-reported | closed when the gate hands its closure to the harvest step |
| **S5 ★** | solvere | 1 | `fire/pass/mod.rs:78-139`; dupes at `join_after_filter.rs:84-113`, `filter_after_join.rs:208-243` | ⭐ The helper's doc header opens **"ONE COPY"** and it is called from exactly ONE of three sites. The full body is still inline at both originals. Fire-hot-path join logic, and a doc comment that lies. | structural | **OPEN** · ✅ I VERIFIED | `grep -rn 'left_activate_join' src/rete/kernel/` → 1 call; both other ranges still hold the body |
| **S6** | solvere | 1 | `filter.rs:231-288`, `filter_after_join.rs:120-166` + `:247-297` | Negation/Exists dispatch triplicated; the one real divergence is `filter.rs`'s `leading_emitted` dedup. | structural | **OPEN** · ⚠ ward-reported | closed by one shared helper |
| **S7** | solvere | 1 | `fire.wat:276,347,509`; `explain.wat:66,92`; `insert.wat:52` | The "unwrap a must-succeed Outcome" 3-arm protocol re-derived at ~7 sites. | incidental | **OPEN** · ⚠ ward-reported | closed by `expect-fired` / `expect-inserted` helpers |
| **S8** | solvere | 1 | `pass.wat:459-502` vs `explain.wat:10-49` | "Derive facts for a token via RHS" walked twice; the two diverge only in what they do with the fact. | incidental | **OPEN** · ⚠ ward-reported | closed by a shared `derived-facts-for-token` |
| **S9** | solvere | 1 | `accum-pass.wat:28-142` | A justified braid MISSING ITS RUNE — forced by wat's invariant parametric types, documented in prose at `:11-16`, but no `rune:solvere(load-bearing-coupling)`. | structural (paperwork) | **OPEN** · ⚠ ward-reported | `grep -c 'rune:solvere' wat/rete/oracle/accum-pass.wat` → 0 |
| **T1 ★** | struere | 1 | `fire/mod.rs:1527`, `:1744`, `:1822` | The join-key family **panics** — 11 sites — where `kernel/outcome.rs` states the law *"a dynamic failure is a value a caller must match, never a raise."* `driver_of` in the SAME FILE returns `Result<_, EvalBreak>` for the identical hazard. | L2 | **OPEN** · ✅ I VERIFIED | `driver_of` → `Result<&CondDriver, EvalBreak>`; the three key fns → 11 `panic!` |
| **T2** | struere | 1 | `fire/acc.rs:13-15` | `rune:struere(host-constraint)` whose reason (*"would duplicate `BindView`"*) is duplication-avoidance, not a missing language mechanism. | L2 | **OPEN** · ⚠ ward-reported | closed when recategorised or the reason names a real host limit |
| **T3** | struere | 1 | `accumulate.rs:222,308`; `filter.rs:161` | The 4-field `BindIntern{…}` literal hand-spelled at three production sites; `FireSession::bind_intern` does this wiring but is `#[cfg(test)]`-gated. | L2 | **OPEN** · ⚠ ward-reported | closed by a production `BindIntern::from_wm` |
| **T4** | struere | 1 | `fire/pass/alpha.rs:76-89` | `ClassPlan::observe -> bool` conflates *"defer"* with *"not a leaf class"*. The codebase cured this exact shape with `OperandSlot`. | L2 | **OPEN** · ⚠ ward-reported | closed by a two-variant enum |
| **T5** | struere | 1 | `fire/pass/mod.rs:162`; `accumulate.rs:59-89`, `filter.rs:60` | `pre_dispatched` is a bare `HashSet` two passes must both honour; the invariant holds only by `delta.rs`'s call ORDER. ⚠ Same site as **I1**, different lens. | L2 | **OPEN** · ⚠ ward-reported | closed by an owned ledger with one door |
| **T6** | struere | 1 | `wat/rete/oracle/fire.wat:57-76` | `walk-filter-ids` dispatches three passes. **Ward's own verdict: no action** — the `rune:intueri(naming)` already covers it. | L2 (self-disclosed) | **CLOSED — no action** · ✅ I VERIFIED | the rune at `:54` is accurate |

## Cast log

| # | target | cast at | wards mustered | returns in `reports/` | L1 | L2 |
|---|---|---|---|---|---|---|
| 1 | `src/rete/kernel/` + `wat/rete/oracle/` | 2026-09-07 | 13 inward + circumspicere last | intueri ✓ purgare ✓ solvere ✓ struere ✓ (in flight: conferre; not yet cast: 9) | 0 | 18 |

## Verified by the orchestrator, not taken

Every row above was re-read against the disk before it was rowed; a ward's verdict is a hypothesis
until a `file:line` confirms it.

⛔ **A ✅ in the status column means I re-read the disk myself. A ⚠ means the row is the WARD's claim,
carried unverified.** Both are rowed, because dropping the unverified ones would hide work; but only
the ✅ rows may be cited as fact. This distinction is the whole reason the status column exists.

- **I1** — CONFIRMED. `accumulate.rs:59` clears `pre_dispatched`, `:76` calls `dispatch_where_tests`,
  `:88` inserts into it. The second responsibility is real and the doc at `:18` does not name it.
- **P1** — CONFIRMED. `grep -rn 'retain-supported'` over `wat/ wat-scripts/ wat-tests/ tests/ src/`
  returns three hits, ALL inside `fire.wat`: the doc header `:231`, the `defn` `:242`, and a
  self-reference `:327`. No call site anywhere. ⚠ And it is MY leftover — the factbag-one-owner
  strike I drew this morning replaced it with `factbag::retain` and did not delete it.
- **S5 ★** — CONFIRMED, and worse than the row alone conveys. All three sites carry the identical
  `keyed_join_persistent` / `FilterJoinIdx` / `FireCtx` / `record_tokens` shape, and
  `grep -rn left_activate_join src/rete/kernel/` finds ONE call (`filter_after_join.rs:75`). The
  helper's doc header at `mod.rs:78` opens with the words **"ONE COPY."** The extraction described
  in that header was never applied to the two sites it names.

## Runes weighed

| rune | verdict |
|---|---|
| `rune:intueri(naming)` at `wat/rete/oracle/fire.wat:54` — *"the name is the historical walk-sorted-ids split, not filter-alone"* | **CLEAR, confirmed.** `walk-filter-ids` does dispatch `accumulate-pass`, `filter-pass` AND `hash-join-pass` (verified over `:57-76`). The rune names the mismatch honestly and gives a checkable cost for the rename. |

## Convergence, per ward

| ward | verdict |
|---|---|
| intueri | 0 L1 + 1 L2 — **DIVERGES** (narrowly) |

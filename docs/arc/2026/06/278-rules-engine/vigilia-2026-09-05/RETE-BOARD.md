# RETE-ONLY BOARD — vigilia 2026-09-05, re-scoped

> **Builder's ruling, 2026-09-05:** *"we should only be working on rete here… main is doing a lot of
> cleanup across the code base… this session is dedicated to making the rete code an exemplar."*
>
> `WORK-LIST.md` holds all 81 L1 / 97 L2 across the whole substrate. **This file is the subset that
> is rete.** Everything not listed here belongs to main. Every row below was grounded at HEAD
> `5924f664b` when this file was written.

## ⛔ RE-GROUNDED 2026-09-07 at HEAD `b6ffdff1d` — SEVEN ROWS SAID **OPEN** AND ALL SEVEN WERE CURED

Every DRIVEN and CLASS A row was re-checked against the source this session, one at a time. **F1,
F2, D3, D2p, D1, A8, A3 were all cured; A4 was cured on its rete half.** Not one had been closed
here. A recovering instance reading this board on the morning of 2026-09-07 — and the *Recommended
order* below told it to start with F1 — would have gone to strike a corpse.

**The mechanism, and it is the one the recovery file catalogs as FM 30: a row's status was written
in TWO places and re-derived in NEITHER.** `WORK-LIST.md` carries the same rows, also OPEN
(`:100` A8, `:153` D3, `:188` F1). Both files are hand-maintained claims; nothing re-runs them.
The cures were real, landed, floored and scored — the *record* is what rotted, and it rotted
silently because a markdown cell has no failure mode.

**Why the existing gates could not see it.** `no_stale_path_in_doc.rs` checks that a cited *path*
resolves — but every one of these seven paths still exists; what vanished was the **symbol**
(`any_mixed` deleted, `beta_written`'s four bypass sites gone, `PersistentMap/keys` replaced by
`topological-node-ids`). `rete_citation_resolves.rs` does resolve symbols, but its population is
comments under `src/rete/` — `docs/` is walked by nothing. The class is blind to both **by
construction**, which is the same argument `rete_citation_resolves.rs:13-28` makes about itself.

### ⭐ AND THE RULE AGAINST THIS WAS ALREADY WRITTEN DOWN — ONE FILE AWAY, FOUR DAYS EARLIER

`WORK-LIST.md:10-13`, in this same directory:

> *"⛔ **STATUS IS EDITED HERE, IN PLACE. Never append a closure below a row.** A row's status living
> in two places IS the defect — `exigere` found exactly that in this arc's TRACKED DECISIONS.
> **One row, one place.**"*

This board shipped with its own `status` column anyway — **the second place its neighbour forbids** —
and both copies then rotted in lockstep, which is precisely the failure that rule predicts. It is
**the arc's signature shape for the tenth time, and the first time in the RECORD rather than the
code**: the tree had already written the correct rule down and applied it in exactly one place.

**So status is now collapsed, not merely corrected.** Every TABLE ROW below points at
`WORK-LIST.md`, which is the sole authority; the closure citations live there. **This file declares
SCOPE and holds DRIVEN EVIDENCE — its tables do not carry status**, so there is no second copy left
to rot. ⚠ The prose sections *after* the tables (census, cost, spec-vs-code) summarise GROUPS of
rows and still read `✅ CLOSED`; they are narrative, their per-row authority is likewise
`WORK-LIST.md`, and collapsing them is the remaining half of this cure. A rung-1 fix would
have been to rewrite these cells truthfully and date them; that only resets the clock.
`[[an-inherited-work-list-is-a-claim]]`

## Out of scope — do not work these here

CLASS Ω (CLI / `distribution/`), CLASS B (doc-comment enumerations, mostly `runtime.rs`/`check.rs`),
CLASS C (the `:wat::` vocabulary, control chars, signing), CLASS E (unowned trees, `examples/`,
`.wat.bad`), all of `partire`'s splits, `conformare`'s error types, `secare`'s io_uring, `purgare`'s
`ChildHandle`, `intueri`'s doc comments, `exigere`, `excusare`. **Ω4 is halted — see
`../strike-config-silent-setters/HALTED.md`.**

Already landed this session and outside scope, kept because reverting green work is worse:
`strike-mode-parity` (`src/distribution/`).

---

## DRIVEN — reproduced by execution, not read

| id | site | what | status → the one authority |
|---|---|---|---|
| **F1 ★** | `wat/rete/oracle/explain.wat:10-49`, doc at `:53` | **`fire-rules-explain$oracle` is NONDETERMINISTIC.** 8 samples, 8 producing rules: native stable `vex::aaa` 8/8; oracle returns **four** distinct rules, agreeing 2/8. Single-producer control stable on both. `harvest-support` folds `(:wat::core::PersistentMap/keys network)` — HAMT order, **no sort** — while its own doc claims *"First-producer-wins, matching the native index."* The sibling `wat/rete/oracle/fire.wat` has four **sorted-ids** walkers and states the law. **The referee for explain is not a function**, and the differential compares only `PersistentMap/length`. | → **`WORK-LIST.md` F1** |
| **F2** | `wat/rete/oracle/insert.wat:100` | `retract` removes **every** equal fact; `insert` stages one. Driven: 2 identical inserts → 3 facts; 1 retract → 1; the derived consequence vanishes. Documented as "by value equality", so an asymmetry the code states — but the multiplicity loss is real. | → **`WORK-LIST.md` F2** |
| **D3** | `fire/pass/mod.rs:152`; `fire/mod.rs:2090`, `:2100`, `:2124` | **Four `beta_written` sites still bypass `record_token`/`record_tokens`**, and `pass/mod.rs:24-28` claims *"a future site cannot push without counting."* **Mutation-proved invisible**: dropping the census at a bypass → 100 tests, 100 passed; dropping it inside the door → RED. No census world contains a `:where`. A1 did not touch these. | → **`WORK-LIST.md` D3** |
| **D2p** | `src/rete/reachability.rs:1659-1665` | A discrimination row that has **never executed** — the rewrite targets the miss face of a constant, so `replacen` no-ops and the `if` is never entered. Driven: swapping in the `assert_ne!` its two siblings use goes RED on the first iteration. Note `src/rete/mod.rs:86` wraps the file in `#[cfg(test)]`. | → **`WORK-LIST.md` D2** |
| **D1** | `src/rete/kernel/tests/right_index_counter_invariant.rs` | The D2 acceptance test is a **tautology** — `indexed_n[J] == Σ\|buckets[J]\|` holds by construction; verified by grep that nothing mutates those fields outside `session.rs`. One possible outcome. **A1's cure gives the left side the same shape, so this now under-guards both.** | → **`WORK-LIST.md` D1** |

## CLASS A remnants — the D2 shape, still in rete

A1 is CURED (`0ee56325f`) and proved the pattern: **at least some CLASS A rows are unfinished halves
of cures we already shipped.** Check each against that lens before treating it as fresh.

| id | site | pair | consequence | status → the one authority |
|---|---|---|---|---|
| A8 | `fire/pass/alpha.rs:85`, `:139-145`, `:217` | `class_ids` `(Vec<u32>, bool)` + `any_mixed`, two disjoint `&mut` arms | **fact loss**, not duplication. Header says *"THE CURE IS THE `bool` BELOW"* — a bool cure maintained by convention | → **`WORK-LIST.md` A8** |
| A3 | `rete/compiled_cond.rs:219-234`; `export.rs:1415` | `slot_keys` / `output_slots` parallel arrays, hand-checked at one of two writers | a rule silently stops matching; the guard returns `None`, indistinguishable from "did not match" | → **`WORK-LIST.md` A3** |
| A4 | `value/value.rs:848-857`, `:1031-1047`; `fire/delta.rs:188-196` | `identity` memo vs a walk computing a **different function** on miss; `seen_ids`/`seen_rest` are two halves of one set | fixpoint dedup breaks if any cross-variant `eq` arm is added | → **`WORK-LIST.md` A4** |

## Cost — `temperare`, all five in the fire path

No measurement is claimed; each row names its own trip count from a test the ward read.
**Gate the ratio, never the millisecond** — and `temperare`'s own L3: every counter measures
occurrences of a named operation, none measures lookups performed, so all five are invisible to the
gates by construction.

`join_extend` 3 SipHash lookups per emitted pair (`fire/mod.rs:686-724`) · `root_join_delta` 5 map
ops per element on keys fixed by the enclosing loops (`pass/root_join.rs:59-79`) · `production_delta`
an `entry` per derived fact with the identical hoist documented 100 lines away
(`pass/production.rs:124-127`) · `key_of_el`'s `col_field_of` hoisted in two places and in none of
the three `hash_join.rs` per-element loops · `ensure_gather` re-deriving its own cache key per token.

✅ **ALL FIVE SCORED — CLOSED 2026-09-07.** `strike-join-extend-hoist` · `strike-root-join-batch` ·
`strike-production-entry-hoist` · `strike-col-field-of-measure` · `strike-combinator-inner-is-it-rare`,
each with a `SCORE.md`. ⭐ The fifth landed **no code**: it was a measurement, and it upheld the
rune it was drawn to challenge — the combinator-inner path runs 206 of 245583 times (0.084%), so
the rune's premise holds and is now **dated** rather than assumed. A strike that changes nothing
and dates a premise is a strike that succeeded. `[[a-metric-without-its-instrument-cannot-be-rechecked]]`

## Instruments — the census names its own quantities wrong

The `recon/census-name-audit.md` sweep returned **13 sections (A–M)** of *the counter's name says X,
the quantity is Y*, all in the rete census: `filter:test-pass` counting passes ∪ elided-passes ·
`match:calls` counting calls that had a pattern · `prod:vec-alloc` carrying a hardcoded ×2 ·
`dbeta:alloc` a 0/1 flag under an allocation name · `seed:mixed-class-activate` class-shaped, per-FACT.
**Every performance claim in this arc rests on these.** Unrowed until now.

✅ **CLOSED 2026-09-07 — the census was worked to completion, A through M.** The useful axis turned
out not to be *"is the name wrong"* but **what, if anything, is watching the number**, which
produced four renames, one split, one deletion, one armed tripwire, one gate, one no-op, and two
strikes that correctly produced no mutation proof and said so. Two of the thirteen audit rows did
**not** survive contact (K/L's *"neither can emit a 0 row"* — CATCHUP can; M's `filter:test-pass`
second key — no such call exists); eleven did. ⚠ One row from this section is deferred, not closed:
census G's **`emitted ⇒ ever read`** gate. The census-name lint enforces read ⇒ emitted; the reverse
is enforced by nothing, which is how three `prod:` counters reached HEAD unobserved.

## Spec-vs-code, rete only

`conferre` L2-1..3: a leading accumulate re-seeds every round into a cumulative beta while leading
`:not`/`:exists` does not · ~~`insert-all`'s hardcoded `OP` defeats the stated reason its checker is
parameterised~~ · stratify's `+1` for `:exists` / accumulate-`:from` over a derived type diverges
between native and oracle.

- **L2-2** ✅ **CLOSED 2026-09-07** (`e95b5ba33`) — `insert_facts_on_session` now takes
  `op: &'static str` and each entry point passes its own verb. ★ The arc's signature shape for the
  ninth time: *the checker took `op` **specifically** so it would report the verb the user wrote,
  and its one caller fed it a constant.* Both arms gated by
  `tests/rete/probe_arc278_insert_reports_the_verb.rs`, which reads the structured
  `RuntimeErrorKind::TypeMismatch { op, .. }` field. ⛔ The single-arm version of that test
  **passed over the live defect** — `insert_all_reports_insert_all` was green before the fix.
- **L2-1** and **L2-3** remain OPEN, and neither may be drawn as a defect yet: L2-1's own report
  forbids it until someone fires a leading accumulate into a HashJoin across ≥2 rounds and counts
  rows; L2-3 has **no instrument at all** — nothing in the tree compares strata.

## Carried from A1, deliberately cut there

✅ **`sequi` L2-a — CLOSED 2026-09-07, and the row was stale, not open.** It read: *"the catch-up's
right-index walk pushes the **whole** alpha memory rather than `right[already..]`."* At HEAD
`hash_join.rs` reads `let tail = right.get(already..).unwrap_or(&[]);` and walks `tail`, with the
comment crediting **D1's cure** — *"the mark is a prefix length. Index `right[already..]`."* D1 took
this row with it and nobody closed it. **An inherited work list is a claim; audit before drawing.** D2's protection is now `is_keyed` on the left
type rather than a detached cache, so this is safe to revisit on its own.

`key_and_index` uses `keys.entry().or_insert()`: a second call keeps the original key list while
indexing with the caller's `key_of_tok`. Idempotent across today's two callers; a third caller
computing a different list would be hidden rather than surfaced. A `debug_assert` closes it.

---

## Recommended order

⛔ **THE ORDER BELOW IS SPENT — every row it names is CURED.** Kept verbatim as the record of what
this board recommended, and as the evidence for why a recommendation must be re-derived against the
tree before it is followed. A recovering instance that trusted it on 2026-09-07 would have opened
`explain.wat` to add a `sort` that has been there since the F1 cure.

> ~~**F1 first.** It is driven, it is the *referee* for explain being nondeterministic, and the fix is
> one `sort` at the check rung with a shared `topological-node-ids` verb at the shape rung — the law
> is already written down in the sibling file. Then **D3** (the false structural claim, mutation-proved
> invisible), then **A8** (fact loss), then the census names, which everything else's numbers depend on.~~

**What is actually open is the STILL OPEN block of the breadcrumb**
(`../CURRENT-STATE-annihilate-interpretation.md`), which is re-derived every session:
`conferre` L2-1 · `conferre` L2-3 · census G's deferred `emitted ⇒ ever read` gate · Stone K's
`benches/` relocation. **L2-1 and L2-3 both need a MEASUREMENT before a cure can be drawn** — each
row's own report says so.

---

## F2 — RESOLVED INTO TWO ROWS, 2026-09-06. Clara driven as the reference.

Builder's ruling on the hierarchy: **Clara is the reference for correctness; the wat oracle must be
in parity with Clara; wat-native must adhere precisely to the oracle's public behaviour, differing
only in performance.**

Clara 0.24.0 driven this session (`insert F` ×2, `insert G`, fire; then retract F once, then again):

```
after 2x insert F + 1x G, fire   : Out rows = 2
after retracting F ONCE          : Out rows = 1
after retracting F a SECOND time : Out rows = 0
VERDICT: one retract removes ONE copy (multiset semantics)
```

wat, same shape, same session: `facts_after_two_identical_inserts=3`,
`facts_after_one_retract=1`, `seen_rows_before_retract=1`.

### ✅ Divergence 2 — derived-fact multiplicity. **JUSTIFIED. NOT A DEFECT. CLOSED.**

Clara derives **2** `Out` rows from two identical `F`s; wat derives **1**. This is deliberate and the
reason is written down in the oracle itself:

- `wat/rete/oracle/fire.wat:214-215` — *"the dedup guard is the **termination invariant** — if a
  derived fact is already in facts, re-adding it would grow facts every round and spin the fixpoint
  forever."*
- `:255-256` — *"the GROWING half: re-run the full match over a **dedup-growing fact set** until a
  round adds no new fact (**monotone-finite termination — datalog property**)."*

wat-rete is a **pure replay-to-fixpoint** engine; set semantics on derived facts IS its termination
proof. Clara is incremental RETE with truth maintenance and needs no such invariant. **Native
(`seen_insert`) and oracle (`merge-facts`) agree**, so the hierarchy holds — this is a considered
departure from the reference, not drift. `fire.wat:265` already reads *"Measured (Clara 0.24.0 is the
authority)"*, so the tree was weighing itself against Clara when it made the choice.

**Do not "fix" this.** Making derived facts a multiset removes the termination argument.

### ⛔ Divergence 1 — `retract` cardinality. **STILL A DEFECT.**

`wat/rete/oracle/insert.wat:100` drops **every** equal fact; Clara drops one. The termination
invariant above does **not** reach it: that invariant is about derived facts *growing*, and retract
shrinks.

**And the tree contradicts itself.** `fire.wat:234-236`, on `retain-supported`:

> *"⛔ IT MUST NOT DEDUP. `insert$oracle` never dedups, so a caller that stages the same fact twice
> **genuinely holds it twice**… Collapsing here would silently retract a duplicate the INPUT
> contains — **a retraction with no cause**."*

Input multiplicity is protected in one function and destroyed in another ~400 lines away.
`retract`'s own docstring claims *"**Symmetric with insert**"* and *"value-precise"* — insert stages
one; retract removes N.

**Four questions:** Obvious NO (the doc implies one-for-one) · Simple NO (one name, two operations) ·
Honest NO (contradicts its own "symmetric" claim and the tree's multiset rule) · UX not reached.

**Direction is fixed by the hierarchy**: retract removes ONE occurrence. Before drawing it, count how
many of the **24 call sites** ever have duplicates in flight — most retract singly-inserted facts,
where both behaviours coincide, but `wat-scripts/perf/grid/where-exists.wat:91-92` nests two retracts
and the grid axes would need re-verifying.

⚠ `experiri` ranked F2 as L2 on the phrase *"by value equality"*. Reading the whole docstring inverts
that: *"symmetric with insert"* is a claim the implementation breaks. **The phrase was documentation;
the paragraph was a contract.**

### ⛔ Why THREE instruments are blind to the retract defect — 2026-09-06

Builder asked whether wat-gen should have caught it. It cannot, and the reasons compound.

**1. The grid fixtures stage no duplicate.** Three axes call `retract`; not one inserts the same
fact twice. `where-not-fact` row 5 is *named* `partial-retract` and uses two DISTINCT values.
(The grid's *comparison* is honest — `run-axis.sh:283` is a literal string compare of `:derived`
after stripping wat's PV tag, and its header says it fails on missing/extra/**reordered**. The
fixtures are the gap, not the instrument. But every Clara side answers `(count (set …))`, which
would collapse a multiplicity difference even if one were staged.)

**2. The port check compares a verb to itself.** `retract` has NO native implementation — no
`RETE_OPS` row, no `fn retract` in `src/rete/`, one definition at `insert.wat:100`. Native and
oracle call the SAME code, so `oracle == native` always. `check-grid-three-way.sh`'s header names
this class exactly: *"a flaw the oracle and its faithful Rust port SHARE is invisible."*

**3. ⛔ CORRECTED 2026-09-06 — the TMS fuzzer has NO MODEL. It is blind for reason 2, not a third reason.**

I asserted a model here after reading the file's COMMENT and never reading its code. Driven:
`final-facts` appears **exactly once in the file — inside that comment**; it is not a function.
`tms::step` (`:68-87`) calls the REAL `:wat::rete::retract` at ops 3/4/5, and `run-prog` folds it
for all four arms (native/oracle × interleaved/one-shot). **Every arm shares the verb**, so the
cure moves all four together and the property (path independence) holds under either semantics.
So there are **two blinding mechanisms across three instruments**, not three: fixtures that stage
no duplicate (1), and a shared verb compared to itself (2) — which covers the port check AND the
fuzzer. `[[a-named-counter-proof-is-still-a-claim]]`: I read what the test SAID, not what it CALLS.

**And the comment is its own L2:** it describes a `final-facts` replay that does not exist,
asserting semantics for a model the file does not contain. Stale prose claiming a proof.

The quoted comment, kept because the author's REASONING is still the interesting part:
`wat-tests/rete/differential-fuzz-tms.wat:26-30`:

> *"THE MODEL IS A MULTISET, NOT A SET, and that is load-bearing. `insert` appends and may
> duplicate; **`retract` removes EVERY fact equal to its argument**. So a program that inserts A0
> twice leaves two facts… `final-facts` below replays the program with exactly those semantics."*

And `:57` shows the author met the behaviour and routed around it:

> *"Two A's so a retraction can leave the class non-empty (**an all-or-nothing retraction cannot
> tell "removed one" from "removed the class"**)"*

`A0` and `A1` are distinct **because** all-or-nothing retraction destroys discriminating power. The
fuzzer mirrors the engine's own split — multiset on insert, all-or-nothing on retract — including
the half that is wrong.

**The lesson, and it generalises past this row:** a property test whose model is read off the
implementation cannot falsify the implementation. The oracle protects against a port bug; only an
EXTERNAL reference protects against a spec bug. That is the whole content of the builder's
hierarchy — Clara → oracle → native — and F2 is the case that proves it: **three instruments, all
green, one external question, immediate divergence.** ⛔ But the *mechanism* is two, not three (see
the correction above) — and a miscounted mechanism is how a cure gets aimed at the wrong instrument.

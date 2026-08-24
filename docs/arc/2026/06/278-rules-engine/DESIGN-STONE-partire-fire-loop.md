# DESIGN-STONE — partire the fire loop

> **Origin (2026-08-24).** The theater hunt exhausted what a timer can
> see, so the code was READ instead
> (`NEXT-STRIKES-theater-hunt.md` § READ, not measured). Builder:
> *"if the code is not an exemplar, then it is to be attacked… the term
> 'workaround' does not share company with excellence."*

## The enemy

```
fire_fixpoint_delta_armed   src/rete/kernel/fire/delta.rs:220-1994
    1774 lines        87% of the file
    12 levels         deepest brace nesting
    16 mutable locals declared at the top level
    9 passes          braided into one body
```

Against the four questions it fails **Obvious** (no reader holds 1774 lines
and 12 levels) and **Simple** (nine concerns, one hat). Those two must hold
before UX is even weighed, so no amount of good naming inside rescues it.

The seams are **already drawn by the authors**:

| lines | size | pass |
|---:|---:|---|
| 220–366 | 147 | prologue: transient, seen-sets, arm binding, scratch |
| 383–507 | 125 | 1. alpha delta |
| 508–562 | 55 | 2. root-join delta |
| 563–960 | **398** | 3. hash-join delta |
| 961–1198 | 238 | 3.25 accumulate |
| 1199–1398 | 200 | 3.5 filter (Test / Negation / Exists) |
| 1399–1483 | 85 | 3.6 join-after-filter |
| 1484–1690 | 207 | 3.7 filter-after-join |
| 1691–1794 | 104 | 4. production |
| 1795–1883 | 89 | A8 census |
| 1884–1901 | 18 | 5. terminate — **NOT a pass**: owns the loop `break` |
| 1902–1984 | 83 | epilogue: cardinality census, OUT |

## What this stone does NOT claim

**It does not claim to remove the workarounds.** The read-verdict first said
narrowed borrows would dissolve "several" of the five borrow-checker clones.
Probing that before briefing showed it is **one of five**:

- `:1250` (Exists) is `wm.alpha` against `wm.bind_pool`/`i64_by_fact`/
  `bind_only`/`cond_key_ids` — **distinct fields**, so a field-split borrow
  removes it. That is theater-hunt **T2**, and it does not need this refactor.
- `:1000` and `:1224` read `d_beta[parent]` and write `d_beta[child]` — the
  **same HashMap**, and a round-local rather than a `wm` field. A function
  boundary changes nothing.
- `mod.rs:510` (`token_assoc`) reads and writes the same pool.
- catch-up take-left is the same beta map.

Those four need their own techniques (disjoint-key access, a two-phase
collect, `extend_from_within`, a restore guard) and are **separate strikes**.
This one is justified on craft alone, and it must be, or it will be justified
on a benefit it cannot deliver.

**It does not claim a speedup.** Extraction is behaviour-preserving. The gate
below requires the grid NOT to regress; a win would be a surprise and is not
predicted.

## The algorithm

A new `src/rete/kernel/fire/pass/` module, one file per pass. Each pass takes
a `RoundCtx` carrying the round-local state (`d_alpha`, `d_beta`, the caches,
the scratch buffers) plus `&mut FireSession` and the immutable `&arm.*` views.

```
fire/pass/alpha.rs        fn alpha_delta(ctx: &mut RoundCtx) -> Result<(), EvalBreak>
fire/pass/root_join.rs    fn root_join_delta(ctx: &mut RoundCtx) -> …
fire/pass/hash_join.rs    fn hash_join_delta(ctx: &mut RoundCtx) -> …
fire/pass/accumulate.rs   …
fire/pass/filter.rs       …
fire/pass/production.rs   …
```

`fire_fixpoint_delta_armed` becomes prologue + a round loop that calls nine
named things + epilogue — the shape the section comments already describe.

**`RoundCtx` is a struct of borrows, not a new owner.** It must not copy or
re-own any memory; if a field cannot be borrowed into it, that pass keeps its
argument list explicit instead. A context object that quietly clones would
replace a readability problem with a performance one.

## ★ THE ONE CONTRACT DECISION

**Each pass moves out whole and unchanged.** No clone removed, no name
improved, no comment rewritten, no `?` added or removed in the same commit as
a move. A move whose diff is only a move can be reviewed by reading the diff;
a move that also fixes things cannot, and this is a 1774-line function where
"cannot be reviewed" is how a silent behaviour change ships.

Improvements land in **follow-up** commits, each with its own gate.

## The gate

Per pass, every one of these before the next pass starts:

1. `cargo nextest run --release -E 'binary_id(wat::rete)'` — 363/363, and the
   differential `spec_equals_native_on_every_where_family` green. **This is
   the load-bearing gate**: the oracle is a whole second implementation of the
   same semantics, so a behaviour change during extraction shows up here.
2. `differential_three_stratum_negation` 3/3.
3. `probe_arc278_concurrent_retes` — 48 concurrent engines still agree.
4. `scripts/floor.sh` GREEN. On a red: **do not re-run**, capture the ARM.
5. Clippy `--release --workspace --all-targets -- -D warnings` silent.
6. Commit and push `grok-rete` — the DR site, per save point.

At the end of all nine, and only then: full grid `GRID_SKIP_ORACLE=1
GRID_RUNS=3`, 30/30 `:match`, and no axis regressed beyond its noise band
(±1% on a big cell; sub-2ms cells are unreadable — see the noise-floor table).

## Predicted outcome

Written before the work: **no measurable perf change** in either direction.
`fire_fixpoint_delta_armed` drops from 1774 lines to roughly 150 of
orchestration; no pass exceeds ~400 lines and most land under 250; max nesting
falls from 12 toward 6–7. One clone (`:1250`) becomes removable, in its own
later commit.

If the grid moves beyond noise in EITHER direction, something other than a
move happened — stop and find it.

## Blast radius

`src/rete/kernel/fire/delta.rs` and a new `src/rete/kernel/fire/pass/`.
No `.wat`. No Session field. No public surface. No type changes. The oracle is
untouched by construction — it is a different implementation and the
differential is the gate.

## Out of scope = REJECTED

- Fixing any of the four same-container clones during the move. Separate
  strikes, named above.
- Renaming anything. The names read well; `intueri` had no finding here.
- Touching `src/rete/kernel/tests.rs` — `partire` returns **LEAVE** on it
  (VIGILIA-LOOP standing orders).
- Changing the census marks, or their nesting. The instrument's own tax is a
  known, recorded distortion; moving marks mid-refactor would make every
  before/after in the arc incomparable.
- A perf justification for this stone. It has none and must not acquire one.

## Method note — how the alias re-spelling is done (added 2026-08-24)

Each pass reads prologue aliases (`kind_ids`, `beta_readers`, `compiled_conds`,
`feeding_alpha_of`, …) that are just `&arm.<field>`. The move re-spells them to
`arm.<field>`, and on pass 3.6 that was done with a blanket substring replace,
which corrupted a struct-field **shorthand**: `compiled_conds,` inside a
`FireCtx { … }` literal became `arm.compiled_conds,`, which is not valid
shorthand, and then needed `compiled_conds: &arm.compiled_conds,`.

The compiler caught it, as it caught a missed alias (`feeding_alpha_of`), a
re-borrow, a redundant field name and an unused `sym` parameter — five errors on
one pass, none of which could reach a commit. That is the wall working, and it
is why the gate is `-D warnings` rather than a reading.

Still, the method is sharpened for the remaining passes: re-spell on **word
boundaries**, not substrings, and check struct literals for shorthand before
replacing. An extraction that needs five compiler round-trips is not wrong, but
it is slower than one that needs none.

**Settled on pass 3.5 — do not re-spell the aliases at all.** Re-declare them
inside the pass, exactly as the fire prologue declares them:

```rust
let kind_ids = &arm.kind_ids;
let compiled_conds = &arm.compiled_conds;
…
let RoundScratch { match_scratch, .. } = scratch;
```

The moved body then needs **zero** re-spelling, so shorthand cannot break, the
signature stays at six parameters instead of fourteen, and the diff is a move by
construction rather than by careful editing. Pass 3.5 built clean on the first
attempt with this pattern, against five round-trips for 3.6 without it.

**And beware the scan itself.** `\bbind_only\b` matches inside `wm.bind_only`,
so the pre-scan reported two round locals the filter pass never touches; both
were destructured and both came back as `unused_variable`. A name test that
ignores the receiver will over-report every time a session field and a round
local share a name — which here they deliberately do. Exclude a preceding `.`.

## Sequencing

Smallest and most isolated first, so the pattern is proven cheaply before the
398-line pass:

1. **2. root-join** (55) — the probe. If `RoundCtx` cannot carry this one
   without cloning, STOP: the design is wrong and the rest is not attempted.
2. ~~**5. terminate** (18)~~ **STRUCK FROM THE SEQUENCE 2026-08-24** — it is
   not a pass. Those 10 lines are the loop's own epilogue and they own the
   `break`; extracting them would force a control-flow reshape (return a bool,
   break at the call site), which is exactly the change a move commit forbids.
   The section comments name nine sections; only eight are passes. Recorded
   rather than silently skipped, because the map was wrong, not the work.
   Then **A8 census** (89).
3. **4. production** (104) — the pass with the most unapportioned time; moving
   it whole makes its interior readable for the first time.
4. **3.6 join-after-filter** (85), **1. alpha** (125).
5. **3.5 filter** (200), **3.7 filter-after-join** (207).
6. **3.25 accumulate** (238).
7. **3. hash-join** (398) — last, because it is the largest and carries the
   catch-up take/restore invariant.
8. Full grid. Then the four workaround strikes, each on its own stone.

**STOP-1:** if any pass cannot be extracted without cloning state that is
currently borrowed, halt and surface it — do not invent a clone to make the
move compile. That would be trading the exact thing this attack is for.

**STOP-2:** if the differential goes red at any step, revert that pass's
commit rather than debugging forward. Each pass is one commit precisely so
that revert is cheap.

## Weigh (2026-08-24) — LANDED, all eight passes

| | before | after |
|---|---:|---:|
| `fire_fixpoint_delta_armed` | **1774 lines** | **657** (37%) |
| max brace nesting | **12** | **8** |
| `delta.rs` | 2043 | 937 |
| passes in one body | 9 sections / 8 passes | 0 |

`fire/pass/`, none over 449 lines including its header: hash_join 449 ·
filter_after_join 250 · filter 243 · alpha 178 · production 138 ·
join_after_filter 121 · round_census 113 · root_join 88.

**Per-pass gate, eight times, no exceptions:** rete cohort 363/363 including
the oracle differential `spec_equals_native_on_every_where_family`;
`differential_three_stratum_negation` 3/3; `probe_arc278_concurrent_retes` 5/5;
clippy `--release --workspace --all-targets -D warnings` silent; floor GREEN
(4942 passed, 19 skipped, no ARM). Eight floors, eight greens, zero reds.

### The perf prediction held, and no win is claimed

Grid `T03-58-45Z`, 30/30 `:match`, 30/30 `:us`. Big cells against the
pre-strike baseline: strat-neg `[6 2000]` −0.9%, accum `[200 200]` −0.5%,
fanout `[40000]` −2.2%, deep-cascade `[50 100]` −3.8%. Nothing regressed.

The stone predicted **no measurable change in either direction** and forbade
this refactor from acquiring a perf justification. It does not get one now: the
consistent small negative drift is as easily explained by the pre-strike
baseline being a high run as by anything the moves did, and every one of those
cells sits inside its historical range. **The claim is "no regression", not
"a speedup".**

A mid-strike grid (after three passes, at the builder's request) caught fanout
`[40000]` at +4.3% — the exact shape an inlining loss would take, on the axis
where the just-moved `production_delta` runs 40k times. It did not reproduce at
RUNS=5. Running that check at the midpoint rather than only at the end was the
right call: had it been real, finding it after eight commits would have meant
bisecting eight instead of looking at one.

### What the strike actually cost, honestly

**Nine dead prologue aliases** were retired — `compiled_rhs_cache`,
`test_sibs_of`, `compiled_drivers`, `compiled_wheres`, `where_tree`,
`test_children`, `feeding_alpha_of`, and two mis-destructured scratch fields.
**Not one was found by reading.** Every one came from `unused_variables` after a
move made it dead. The 1774-line function was hiding its own dead bindings by
being too large for anyone to notice them — and this stone claimed nothing about
dead code.

**The method changed twice, mid-strike, and both changes are recorded rather
than smoothed over:**

1. Passes 1–4 re-spelled prologue aliases to `arm.<field>` in the moved body.
   That broke a struct-field shorthand on 3.6 and cost five compiler
   round-trips. From 3.5 on, the aliases are **re-declared inside the pass**
   exactly as the prologue declares them — body untouched, shorthand cannot
   break, signature six parameters instead of fourteen. Pass 3.5 then built
   clean first try.
2. `RoundScratch` was introduced at pass 1 (alpha), not up front — precisely
   where the stone said the decision belonged. Root-join needed 5 parameters,
   production 7, alpha's seed path **14**. Borrows only, constructed inline so
   its lifetime ends with the statement, destructured on entry so bodies keep
   their inline names.

**Two errors only the move-verifier could see.** The compiler was green for
both: a doc contradiction in `merge_facts`, and — in this final pass — my own
`&x[i]` restoration rewriting **sixteen comment lines** of the P6 algorithm
prose, turning `dl = d_beta[P]` into `dl = &d_beta[P]` in the explanation of the
whole pass. Checking a move mechanically instead of reading the diff and
believing it is what caught both.

### What is NOT done

- **`terminate` was never a pass** — 10 lines owning the loop's `break`. Nine
  sections, eight passes; the map was corrected, not the work.
- **The take/restore invariant still holds by convention.** `hash_join.rs`
  documents it: one take, two restores, one of them twelve levels deep, exactly
  one early exit and it restores first. A future `?` in that window silently
  drops a beta memory. A guard shape is a named follow-up.
- **Four of the five borrow-checker workarounds remain**, as this stone said
  from the start. Only `:1250` (Exists, disjoint fields) is a field-split; the
  others are same-container conflicts needing disjoint-key access, a two-phase
  collect, and `extend_from_within`. Separate strikes.
- **The `production` phase mark still spans wider than the pass it names** —
  it encloses the A8 census. Moving it is a census-tree change; every
  `production` number in the arc reflects the wider span.

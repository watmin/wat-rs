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
| 1884–1901 | 18 | 5. terminate |
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

## Sequencing

Smallest and most isolated first, so the pattern is proven cheaply before the
398-line pass:

1. **2. root-join** (55) — the probe. If `RoundCtx` cannot carry this one
   without cloning, STOP: the design is wrong and the rest is not attempted.
2. **5. terminate** (18) and **A8 census** (89).
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

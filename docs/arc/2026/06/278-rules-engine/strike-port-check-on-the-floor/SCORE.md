# SCORE — the port check was already there, and it compared `X == X`

> **Written after the orchestrator's own re-run.** Every number below was measured on this machine
> at HEAD `daa92c3b0` + the strike, or re-derived by command. The rider's figures are noted where
> they differ; none of them was taken on trust.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 | ★ the port check runs on the floor | ✅ `every_grid_axis_native_matches_its_oracle`, **PASS 12.2 s**, 12 axes |
| 2 | ★ the corpus can express D7's shape | ✅ **185 `defrecord`s / 0 parametric → 186 / 1**. `parametric-erasure.wat:54` carries `(:wat::core::defrecord :pe::Box :- [T] [k <- :wat::core::i64  v <- :T])` |
| 3 | ★ the new axis REDs on D7's defect | ✅ **re-driven by the orchestrator** — see below |
| 4 | the gate reads the ORACLE column | ✅ rider's mutation 2 |
| 5 | failure names both sets | ✅ both sets in full, plus the symmetric difference |
| 6 | `fanout` covered | ✅ covered, 400 elements at `[500]` |
| 6b | ★ every set non-empty | ✅ **mutation-proven by the orchestrator** — see below |
| 7 | all axes green at HEAD | ✅ **12/12**, every set non-empty |
| 8 | runtime ≲ 60 s | ✅ **12.2 s** on the floor |
| 9 | no `src/` change | ✅ zero diff in index AND worktree; `alpha.rs` md5-verified against HEAD after mutation 1 |
| 10 | floor ≥ 5375 + arms, lints ≥ 228, clippy rc=0 | ✅ **`5376 tests run: 5376 passed, 21 skipped`** (424.3 s), 0 FAIL rows, lints **228**, clippy rc=0 |

## ⛔⛔ THE FINDING — a gate claimed this check and could not have run it

`grid_axes_run_and_derive_nonvacuously` (`tests/rete/wat_scripts_grid_axes_live.rs`) carried a
header asserting `:derived` equals `:oracle-derived`, code that appeared to do it, and a comment
that said in its own words: *"The data was being computed and discarded; **this reads it**."*

**It could not.** `run_sized_axis` calls `skip_oracle_fire` first, which rewrites the
`(:wat::rete::fire-rules$oracle staged)` call site to `(:wat::rete::FireOutcome::Fired fired)` —
wrapping the **already-fired native session**. So `ofired` IS `fired` and the comparison was `X == X`.

**Driven by the orchestrator**, `min-finding [100 3]`, one substitution applied by hand:

```
UNMODIFIED (oracle really fires)   :oracle-ns 570,431,752
REWRITTEN (skip_oracle_fire)       :oracle-ns       5,519
```

**A 100,000× collapse — the interpreted oracle never runs.** (Rider measured 544,437,493 → 5,608 on
the same substitution; reproduces.)

The rider then drove the consequence: with **D7's cure reverted** and that file restored to
`daa92c3b0` with its comparison intact, the test reports `1 test run: 1 passed` — **green, with a
silent fact-drop live in the engine.**

### The shape, because it is new

**The assertion was correct. The test's own setup rewrote the input out from under it.** The
rewrite is *right* for a liveness test — liveness asks whether NATIVE ran, and the interpreted
oracle costs 100,000× — but it silently voided a value comparison sharing the same helper. The
file's span-overlap guard defended the **parse** against reading one field twice; nothing defended
the **source** against firing one engine twice.

This is C16 — a differential that agrees with corruption by construction — **already resident, and
wearing the name of the thing this strike was drawn to build.** Had the strike landed beside it
without reading it, the tree would carry two gates for one pairing, one true and one false, with
the false one older and better-placed to be trusted.

Cure: the false value comparison is **deleted**; the honest presence + distinct-span checks stay;
the header now states what the test can and cannot assert, and points at the new gate.

## The mutation proofs the orchestrator re-ran

**Mutation 1 — revert D7's cure** (`git checkout 523152b31 -- src/rete/kernel/fire/pass/alpha.rs`,
+4/−155, md5 confirmed changed *before* the run):

```
1 of 12 grid axes FAILED the native-vs-oracle port check (11 axes agreed before the failures below):
  parametric-erasure (size [200]): ⛔ PORT BUG — NATIVE AND $ORACLE DISAGREE.
      only in native:
      only in oracle: 1 2 4 5 7 8 10 11 13 14 16 17 19 20 22 23 25 26 28 29 31 32 34 35 ...
```

The oracle-only keys are exactly the **non-multiples of 3** — the erased, unpackable `Box`
instances. **The other eleven axes stayed green**, which is the proof that the corpus hole was real
and that the parametric shape is what closes it. Restored; `alpha.rs` md5 back to HEAD.

**Mutation 6b — empty both sets** (`negation.wat`, both columns → an empty vector). Under a plain
equality check this reports `match`:

```
negation (size [50]): VACUOUS — native has 0 element(s), oracle has 0. An empty set compares
EQUAL to an empty set and reports agreement while proving nothing. Expected 25: size=[items];
Bad is seeded for even k and Ok fires for odd k — the 25 odd keys in [0,50).
```

Restored; md5-verified against HEAD.

**Mutation 2 (rider)** — corrupt `negation`'s oracle column with a conj'd `999999` → that axis REDs,
`only in oracle: 999999`, native untouched. Proves the gate reads the oracle column.

## Honest deltas

- **The BRIEF's ¶1 — *"Nothing has ever compared them"* — and row 1's *"never run, in 23 grids"*
  were wrong in the way that mattered.** A gate existed that *claimed* to. Neither the DESIGN's
  five-item read list nor the BRIEF mentioned `wat_scripts_grid_axes_live.rs`, which owns the axis
  population, the size table the new axis had to join, and a vacuous copy of the deliverable.
  **That omission is the orchestrator's, and it nearly built the strike's own stated failure mode.**
- **The row's count rotted too.** It says *"every `GRID-*.txt`, all 23"*. Derived by command:
  **47 recorded grids tree-wide, 0 carrying `:oracle-accuracy`.** The claim holds harder than
  written. Per F0 the fix is the command, not a new number:
  `find . -name 'GRID-*.txt' -not -path './target/*' | wc -l`
- **A false claim already in the tree, inherited and then retracted.** `extract_vector_field`'s doc
  says *"`:oracle-derived` CONTAINS `:derived`"* and calls a delimiter load-bearing on that basis.
  **It does not** — the colon is part of the needle; `:oracle-derived` holds `-derived`. Verified:
  `python3 -c "print(':derived' in ':oracle-derived')"` → `False`. The rider wrote the same false
  sentence into its own header before checking, then corrected both sites to say the hazard is
  **latent** (against a future `:spec-derived`) rather than live.
- **The rider's gate first mis-classified D7.** Its cardinality guard ran *before* the pairing, so
  a fact-drop reported as *"either the workload changed or the run is not the one this row
  describes"* — every word a complaint about the test table, for a defect in the engine. Reordered
  so the pairing speaks first, and the cardinality check moved onto the **oracle** column so a
  native defect cannot reach it.
- **`git checkout <sha> -- <path>` STAGES.** So `git diff --stat` reports nothing and a real
  mutation reads as a no-op — the exact false-negative that invalidated a C16 proof two days ago.
  `git diff --cached --stat`, or an md5 against `git show HEAD:`, is what actually confirms it.
  Both restores here were hash-verified, not diff-verified.
- **The gate prints nothing on green.** The per-axis element counts in this SCORE come from a hand
  drive at the same sizes, not from the gate. Left silent-on-green deliberately; noted because the
  BRIEF asked for "the gate's full output".

## What this strike did NOT buy

**C9's Clara half (`oracle` vs `clara`, "the SPEC is wrong") stays open.** It needs the JVM and
still costs hours, and it is the pairing that catches a flaw the oracle and its port **share** —
which the port pairing structurally cannot see. **C9 is not closed.**

Nothing under `src/` was touched, so D7's cure is exercised here only through mutation 1.

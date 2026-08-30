# `kernel/tests.rs` — two wards cast, 2026-08-30. Verdicts recorded, work NOT started.

**Builder's ruling stands: it becomes a `mod` with many files, NOT NOW.** This note exists so the
next hand does not re-cast, and so the corrections below are not lost with the session.

The file: **10,189 lines · 89 `#[test]` · 28 module-level helpers · zero `#[ignore]`** — so every
test in it runs on the release floor.

## Why two wards

`partire` (module boundaries) carries a scope clause: *"Test organization … is `complectens`.
Partire is for production code; it does not decompose test suites."*

⛔ **I FIRST READ THAT AS FATAL AND IT IS NOT.** I claimed the independent-test clause is
*incoherent* for a test file ("you cannot test a test"). It transfers cleanly: **can this region
compile and run without the other's fixtures?** That is decidable from disk. The builder pushed
back — *"i think this file's purpose is partire?"* — and was right. Both were cast, one ward per
worker, text embedded verbatim.

## `partire` → SPLIT, five modules

It ruled the exclusion does not defeat the verdict, on grounds worth keeping: the clause names a
DIFFERENT question (composition, not boundary), partire's question has no other owner in the ward
set, and the decision procedure is checkable here rather than arguable.

| module | what | reason to change | severity |
|---|---|---|---|
| `pass_semantics` | `:12–31`, `:48–54`, `:56–659` — 9 tests | the transient shape of a session's memories | **L1** |
| `arm_lease` | `:2959–3321` + `session_net_id` `:3041`, `SCOPED_WORK_WORLD` `:3175` | the arm intern/lease protocol | L2 |
| `alpha_discrimination` | `:2933–2938`, `:3323–3330`, `:8850–8890`, `:9037–9065`, `:8892–9286` | the alpha-tree / compiled-cond CONTRACT (modules outside `kernel/`) | L2 |
| `fire_cost_census` | everything else — 63 tests | **the instrument**: how a phase mark is calibrated and apportioned | L1 (see correction) |
| `binding_repr_bench` | `:1730–1788`, `:1880–2134`, `:8689–8848` | the binding-key / token-bindings REPRESENTATION decision | L2 |

**Refused cuts** (do not re-propose): splitting `fire_cost_census` by grid axis — all six axes share
one secret, the instrument-subtraction arithmetic in `render_phase_table`; and splitting
`round_trip_*` from the join tests — same secret, two pieces of one module.

✅ **VERIFIED BY HAND — `binding_repr_bench` is the strongest claim and it holds exactly.** All four
ranges contain **ZERO** references to `super::`, `FireSession`, `to_transient`, `fire_fixpoint`,
`alpha_pass`, `startup_from_source`, `freeze_src`, `eval_in`, `scratch_wm`. They build `Value` /
`Arc` / `HashTrieMapSync` directly. **A test region naming zero symbols from its host module is not
part of that module by any reading**, and it is `include!`d into `kernel::tests` for no reason.

⛔ **CORRECTION — ITS LEVEL 1 SEVERITY FOR `fire_cost_census` DOES NOT HOLD AS ARGUED.** The ward
claimed *"roughly 63 are measurement instruments: their assertion is a liveness check on the
instrument itself."* Measured over all 89:

| | count |
|---|---:|
| assertions ALL liveness (`> 0`, "never ran") | **5** |
| no assertion at all | 2 |
| at least one SUBSTANTIVE assertion | **82** |

**The fair version survives: 14 tests are ≥100 lines with ≤2 assertions**, led by
`probe_gap_cost_split` at 282 lines / 1 assertion. That is a real assertion-density concern. It is
not "63 tests padding the floor", and the L1 rating rests on the number that was wrong.

## `complectens` → findings, ranked

1. **`calibrate_mark_ns` is a 9,089-line forward reference.** Defined `:10034`, called from EIGHT
   sites above it, earliest `:945` inside `render_phase_table`. ✅ verified. One function moved is
   the whole fix, and it is the highest ratio in the file.
2. **`time_ns` — one name, TWO contracts.** ✅ verified: 22 nested copies, **7** as
   `time_ns(n, body) -> elapsed/n` (**per iteration**) and **15** as `time_ns(body) -> elapsed`
   (**total**). Two quantities differing by a factor of `n` (20k–300k) under one name.
   ⚠ *Correction to the ward's framing:* it said the difference is "invisible at the call site" —
   the ARITY differs, so it is visible if you look. The lie is in the NAME and in the printed
   number a reader compares across sites.
3. **The instrument is hand-rolled ~25 times.** ✅ verified **37** byte-identical
   `let ms = |ns: f64| ns / 1e6;`, and 13 `of` census-row closures in two incompatible return
   shapes. The cure already exists in-file — `render_phase_table` `:937`, 5 callers — and its own
   doc says why: *"two copies is how one of them silently stops subtracting."*
4. `render_phase_table` and `calibrate_mark_ns` have no isolation proof, and every number in the
   file multiplies through them.
5. **Two `#[test]`s assert NOTHING** — `binding_key_cost` `:1886` (81 ln) and
   `binding_repr_microbench` `:1993` (125 ln). ✅ verified 0 asserts each. They self-declare
   "diagnostic, not a gate" while sitting on the floor. R59: *a green number nothing depends on is
   a claim, not a proof.*

## The convergence is the signal

Neither worker saw the other's findings, and neither was given the prior 47/42 census-vs-gate
reading — **both reached it independently.** They then landed on the same fault from opposite ends:
the `*_split` family, `calibrate_mark_ns`, `render_phase_table`, and the zero-assertion pair.

⛔ **AND THEY ARE NOT ALTERNATIVES.** The censuses are ONE module **and** that module is badly
woven. Splitting does not fix the 22 `time_ns` copies; fixing the weave does not stop instruments
from sitting on the correctness floor. Whoever takes this takes both.

**`partire`'s own practitioner's-call is the cheapest real win and needs no split:** give the
instruments a run profile (`#[ignore]` or a `census` feature) so the release floor counts GATES.
Today it counts all 89, and 5 of them cannot fail.

# `kernel/tests.rs` — two wards cast, 2026-08-30. **DONE the same day (`f98226353`).**

✅ **THE SPLIT IS SHIPPED: 10,189 lines → 13 files, largest 1,676.** The builder lifted the NOT
NOW (*"i dislike large files… each test is doing its own thing, yes?"*) and it went in. This note
is kept as the RECORD OF THE REASONING, not as open work — do not re-cast, and do not re-derive
the boundaries from the ward ranges below without reading the two corrections marked ⛔.

⛔ **THE SECOND-CUT REFUSAL BELOW IS STRUCK — IT FAILED AGAINST THE DISK.** `partire` refused
splitting `fire_cost_census` by axis because *"all six axes share ONE secret — the
instrument-subtraction arithmetic in `render_phase_table`"*. Measured over its 62 tests:
`render_phase_table` is named by **4**, `calibrate_mark_ns` by **8**, and **34 of 62 use no local
helper at all**. The coupling the refusal rested on is not there. A shared helper was never an
argument against splitting: it belongs in the PARENT, which is the rule `tests/mod.rs` already
used one level up. `fire_cost_census` is now nine subject modules.

★ **AND THE HOIST KILLED FINDING #1 STRUCTURALLY.** `calibrate_mark_ns` / `render_phase_table`
moved to `tests/mod.rs`, so the 9,089-line forward reference is **unrepresentable** — a parent is
above every child by construction — rather than merely repaired.

⛔ **THIS WARD IS NOW 3-FOR-3 ON OVERSTATED NUMBERS** (severity "roughly 63" → 5; "invisible at
the call site" → the arity is visible; this refusal). Its CITATIONS held every time; its
AGGREGATES did not. Ground the aggregate before you act on it.

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
| `fire_cost_census` | everything else — 63 tests (**counted: 62**) | **the instrument**: how a phase mark is calibrated and apportioned | L1 (see correction) |
| `binding_repr_bench` | `:1730–1788`, `:1880–2134`, `:8689–8848` | the binding-key / token-bindings REPRESENTATION decision | L2 |

**Refused cuts, as the ward wrote them — the FIRST IS STRUCK (see the header), the second stands:**
~~splitting `fire_cost_census` by grid axis — all six axes share one secret, the
instrument-subtraction arithmetic in `render_phase_table`~~ → **measured false: 4 of 62 tests name
it. Split, and the shared helpers went to the parent (`f98226353`).** · splitting `round_trip_*`
from the join tests — same secret, two pieces of one module. **That one holds:** both live in
`pass_semantics`, which was shipped whole.

✅ **VERIFIED BY HAND — `binding_repr_bench` is the strongest claim and it holds exactly.** All four
ranges contain **ZERO** references to `super::`, `FireSession`, `to_transient`, `fire_fixpoint`,
`alpha_pass`, `startup_from_source`, `freeze_src`, `eval_in`, `scratch_wm`. They build `Value` /
`Arc` / `HashTrieMapSync` directly. **A test region naming zero symbols from its host module is not
part of that module by any reading**, and it is `include!`d into `kernel::tests` for no reason.

⛔ **CORRECTION — ITS LEVEL 1 SEVERITY FOR `fire_cost_census` DOES NOT HOLD AS ARGUED.** The ward
claimed *"roughly 63 are measurement instruments: their assertion is a liveness check on the
instrument itself."* Measured over all 89:

| | pre-split | my re-measure | **`probare`, 2026-08-30** |
|---|---:|---:|---:|
| assertions ALL liveness (`> 0`, "never ran") | **5** | 8 | **24** |
| no assertion at all | 2 | 2 | **2** |
| at least one SUBSTANTIVE assertion | **82** | 79 | **61** |

⛔ **THE TRUE COUNT WAS 26, AND BOTH MY EARLIER NUMBERS WERE LOW.** `probare` was cast at the
suite on 2026-08-30 and counted 26 hollow of 89. My classifier said 10 because it could parse
neither COMPOUND liveness (`a > 0.0 && b > 0.0`) nor COMPLEX-EXPRESSION liveness
(`size_of::<Token>() > 0` — which `probare` named the purest hollow form in scope: a compile-time
tautology). All 26 are now converted (`99bf573df`).

⛔ **AND THE REASON THEY MATTERED IS SHARPER THAN "UNTESTED".** These are COMPARISON benchmarks,
and on a comparison benchmark **every plausible failure makes the measured arm look BETTER**: a
no-op `insert`, a lossy `production_to_pm`, a colliding `identity()`, a degenerate `key_of`, a
phase missing from a sum — each does LESS work and prints as a SPEEDUP. `assert!(x > 0.0)` is not
weak verification there, it is **ANTI-verification**: it stamps a broken arm green while the
number misreports what happened. That is how 26 tests sat on a release floor looking like coverage.

**Three hollowness SHAPES, not one:**
1. liveness on a comparison (the lossy arm is the fast arm)
2. a sum with a silently-missing term — `fire_and_top` ranks a cell CHEAPER when a phase vanishes
3. an unasserted premise stated in the test's OWN NAME — `a0_depth_cost_split_at_equal_work`

⚠ **`probe_gap_cost_split` was my CEREMONIAL conversion** — I turned one unfalsifiable check into
six, and `probare` said so. Redone with engine contracts (`extend_token` produces a 3-binding
token and grows the pool by exactly 3).

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

✅ **CLOSED 2026-08-30 — `partire`'s practitioner's-call is DONE, and not the way it proposed.**
It suggested a run profile for all the instruments. Measurement said otherwise: only TWO are
genuinely un-gateable (`binding_key_cost`, `binding_repr_microbench` — measured effects of
1.0–1.9x, inside runner noise, and this repo bans flakes absolutely, so a threshold there would
manufacture the very thing that ban forbids). Those two are `#[ignore]`d with the numbers recorded
as the reason.

The other EIGHT did not need a run profile and did not need a performance threshold either —
**every one had deterministic structure sitting unasserted beside its liveness check**: exact
census counts, index agreement, phase presence, per-component non-vacuity, apportionment. All
eight are now real gates, each mutation-proven. Floor: 89 -> 87 running + 2 ignored, and the 87
all test something.

**⏭ ALSO STILL OPEN — the weave, untouched by the split:** `time_ns` carries two contracts under
one name (7 sites `elapsed/n`, 15 sites `elapsed`) and `let ms = |ns: f64| ns / 1e6;` is written
37 times. Both are recorded in `fire_cost_census`'s successor modules' shared parent; splitting
scattered these across nine files rather than fixing them, which is worth knowing before someone
reads the new structure as finished.

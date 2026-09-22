# BRIEF — STONE 251.8d-ii (FOURTH DRAW): the bootstrap

**Drawn 2026-09-22 against `main` @ `3af0e5d9d`.** Floor 5968/5968 (unconverted), clippy 0,
census `no STOP-8`. Delta **3**, RECOVERY **0**, via `scripts/replay/delta.sh`.
Predecessors: the three prior `BRIEF-STONE-251.8d-ii-*` and their SCOREs.

## ⭐⭐ THIS DRAW HAS A MEASUREMENT THE OTHER THREE DID NOT

The orchestrator ran the whole thing on `main` before drawing — converted the live stdlib, built,
ran the floor, then restored. **You inherit this; do not re-derive it:**

| | |
|---|---|
| conversion | **64/64**, rc 0 (`git ls-files \| grep -E '^wat/.*\.wat$'`) |
| `cargo build --release` | **exit 0**, 23 s |
| ⭐ **the binary STARTS** | **YES** — ⭐⭐ **no `UnknownNamedType` startup death. 255.12 CURED the blocker that stopped the third draw.** |
| floor | ⛔ **3 747 failed / 5 986**, in **24 s** (vs 320 s) — everything dies fast |
| ⭐⭐ **failures naming `wat.gen/Coord`** | **5 286** |

⛔ **ONE SITE BLOCKS THE ENTIRE STDLIB.** `wat/gen.wat:705`, inside the `record` macro's quasiquote
template:

```
UNCONVERTED : (:wat::core::fn [~cv <- :wat::gen::Coord] -> ~T
CONVERTED   : (wat.core/fn   [~cv :- wat.gen/Coord]    :- ~T
```

The binder **NAME** is `~cv` — properly unquoted, hygienic. **`wat.gen/Coord` is the TYPE.**
Hygiene **Gate E** reads it as a **binder-position literal** and refuses the macro at definition:
`ProgramBodyIntroducesName … quasiquote template introduces literal name 'wat.gen/Coord' in binder
position`.

## ⚠⚠ THE ORCHESTRATOR'S MECHANISM HYPOTHESIS IS **UNCONFIRMED** — ISOLATE IT FIRST

**The plausible story:** 255.11 cured Gate E by teaching the binder scan symbol-spelled `let`/`fn`
heads. Before that, a converted `(wat.core/fn …)` inside a template **was not recognised as a `fn`
form at all**, so its param vector was never walked. Now it is — and the walk treats the post-`:-`
**type** as a name.

⛔ **THAT STORY IS NOT PROVEN. TWO MINIMAL REPROS FAILED TO REPRODUCE IT:**

```wat
;; rc=0 — does NOT fire
`(wat.core/fn [~t :- m/Coord] :- wat.core/i64 1)
```

⭐ **So the first job is to find what `gen.wat:705` has that a five-line equivalent does not** —
the enclosing `(wat.core/let [~@binds] …)`, the `~T` unquoted return type, the `typealias` (not a
record) target, the splice, or something else entirely. ⛔ **Do not build on the hypothesis. Measure
which factor flips it, and SAY WHICH.** If the cause is not 255.11, say that too.

## ⛔⛔ HOW NOT TO FIX IT

**Gate E is a hygiene wall that 255.11 closed for a reason** — it was silent for symbol-spelled
templates, and `[[a default-deny gate that dispatches on a spelling is default-allow for every other
spelling]]`. ⛔ **A cure that makes Gate E ignore types by loosening what it inspects can re-open the
capture hole it exists to prevent.**

⭐ **NON-VACUITY, MANDATORY:** Gate E must **STILL REFUSE a genuine hygiene violation** — a template
introducing a real literal **name** in binder position — in **BOTH** spellings, with the same located
reason. **Build that fixture. It is the row that makes the cure trustworthy.**

## The work

1. ⭐ **Isolate** the trigger (above). Report the discriminating factor.
2. **Cure it** so a TYPE in a converted param vector is not a binder name, without weakening the wall.
3. **Convert live `wat/`** — all 64 paths, listed explicitly. `cargo build --release`.
4. ⭐ **SELF-APPLICATION — still owed, three draws running.** With the converted stdlib compiled in,
   run the converted codemod against a copy of an **unconverted** file and prove it **converts it
   correctly**. ⛔ **"It starts" is not "it works"; "it checks" is not "it runs."**
5. ⭐ **IDEMPOTENCE** — a second pass over converted `wat/`: **0 changes**.

## The gate

- ⭐ **THE MEASUREMENT ABOVE, RE-RUN AND GREEN:** 64/64 convert · build exit 0 · ⭐ **floor GREEN**
  (5968+, not 2 239). ⛔ **That floor is the gate. Nothing else substitutes for it.**
- ⭐ **The converted codemod CONVERTS** (step 4) — shown, diffed.
- **Idempotent** — 0 changes on the second pass.
- Gate E's non-vacuity fixture, both spellings.
- clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- ⭐ **`scripts/replay/delta.sh`** — baseline **3**, ⛔ **RECOVERY non-zero is a STOP** (it exits 9).
- ⭐ **THIS STONE LANDS THE CONVERSION** if the floor is green. ⛔ **If it is not, RESTORE `wat/`
  (`git checkout -- wat/` + rebuild — proven three times) and report. DO NOT FORCE IT.**

## ⛔ Operational traps — each has cost this arc real time

- ⛔⛔ **NEVER `git add -A` WHILE A CONVERSION IS IN FLIGHT. USE EXPLICIT PATHS.** ⭐ **The
  orchestrator did this TODAY and pushed 4 half-converted stdlib files to `main`** (`d9b8a82fa`,
  reverted at `3af0e5d9d`). `[[i_committed_on_a_non_quiescent_tree]]`.
- ⛔ `wat/fix.wat` and the stdlib are **`include_str!`'d**: on-disk edits are invisible until
  `cargo build --release`. **Rebuild between every step and say that you did.**
- ⛔ `git ls-files 'wat/**/*.wat'` returns **31 of 64** — git's `**` does not match `wat/*.wat`.
  **This glob has bitten twice.** Use `git ls-files | grep -E '^wat/.*\.wat$'`.
- The conversion takes **~22 min** for 64 files. Bash caps at 600 s — **background it.**
- A converted-stdlib floor takes **24 s when broken** and ~320 s when healthy. ⚠ **A fast floor is a
  symptom, not a speedup.**

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
  ⚠ **`rete::kernel::tests::harvest_cost::harvest_wrap_split` is a KNOWN-UNSOUND wall-clock ratio** —
  diagnosed at `278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md` (systematic allocator-order
  bias, not noise). ⛔ **If it fires, cite the FINDING and report it — do NOT treat it as a pass, and
  do NOT re-run to clear it.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Fourteen stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Twenty corrections across eighteen
  stones** — and ⛔ **this brief's own mechanism story is explicitly flagged UNPROVEN.** Assume a
  twenty-first.
- ⚠ **Reformat in its own commit, or not at all.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

8d-iii · the terminal "keyword heads illegal" cut (**drawn but NOT ruled** — the Rust-embedded corpus
is **8 447 forms across 363 `.rs` files, 7 099 in `src/`**, and the codemod cannot reach string
literals; that number decides whether it is a stone or an arc) · the 416/shape-B/D sweep ·
`:wat::keyword::canonical-identity` · 255.8's wrong-join hole · `Ngram`'s mixed-dialect collision
(**builder's call**).

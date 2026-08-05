# BRIEF — #57 S6, the law-A migration. **The fence is ARMED and the corpus is screaming.**

> **State when this was written (2026-08-05, all measured this session, none inherited):**
> HEAD `467c1fc5` (law A wired, pushed, floor was 4358/4358 green *before* the fence flip).
> The `where` fence is now flipped **in the working tree, uncommitted** (`wat/rete.wat:~684`).
> With it armed: **floor 4322 passed / 39 failed**, and `check-where-shapes.sh` fails **9 of 9**
> families. Those failures ARE the worklist — R52/R65, *the fire is the worklist*.

## The law, and why every refusal is legitimate

> *"The entire rete query language may only be composed from rete primitives."* — the builder

A `where` now admits only `:wat::rete::` ops. `:wat::core::>` is pure, deterministic **and** total;
it is refused for one reason only — it is not from rete. That is why the refusal carries its own
axis (`Axis::RetePrimitive`) and reads *"'<head>' is not a rete primitive"* rather than a lie about
purity.

## ★ THE MAP IS GROUNDED — 70 `RETE_OPS` rows, read off `src/rete/vocabulary.rs` this session

Do **not** infer a target name. Every mapping below was extracted from the table, not guessed.

| bucket | rule | n | who |
|---|---|---|---|
| **A — MECHANICAL** | `:wat::core::X` → `:wat::rete::core::X`, one twin, pure textual | 55 heads | **codemod** |
| **B — HOLON** | `:wat::holon::X` → `:wat::rete::holon::X` (note: **not** `core::`) | 4 heads | **codemod** |
| **C — JUDGEMENT** | multiple twins; the operand TYPE picks the module | see below | **by hand** |
| **D — NEGATIVE CONTROLS** | ⛔ **DO NOT TOUCH** | 3 sites | nobody |

### Bucket C — the judgement sites, with every twin enumerated

```
:wat::core::=      → :wat::rete::core::{i64,f64,string,bool,keyword,enum}::=      (6 twins)
:wat::core::not=   → :wat::rete::core::{i64,f64,string,bool,keyword,enum}::not=   (6 twins)
:wat::core::first  → :wat::rete::core::{PersistentVector,Vector,List}/first       (3 twins)
:wat::core::>      → :wat::rete::core::{i64,f64}::>       ⚠ NO generic twin exists
:wat::core::<      → :wat::rete::core::{i64,f64}::<       ⚠ NO generic twin exists
:wat::core::>=     → :wat::rete::core::{i64,f64}::>=      ⚠ NO generic twin exists
:wat::core::+      → :wat::rete::core::{i64,f64}::+       ⚠ NO generic twin exists
```

**There is no generic rete comparator and there never will be** — RULED, and the reason is totality,
not style: `eval_compare` raises `TypeMismatch` on incomparable operands, so generic `>` is
**partial**. `i64::>` has no such hole. Monomorphising does not merely speed it up, *it deletes the
domain hole.* Per-type IS the total version.

### ⛔ THE ONE MIXED-TYPE SITE — a deliberate axis, ruled by the builder

`wat-scripts/perf/grid/where-numeric.wat:155`

```clojure
(:wat::rete::where (:wat::core::< ?a 0.5))     ; ?a is i64, 0.5 is f64
```

> *"Mixed types are not allowed in rete — this is a violation of form. We only support
> `:wat::rete::{i64,f64}::>=` and so on. The user will need to explicitly cast their number value
> to a float to be measured as a float."*

⇒ it becomes `(:wat::rete::core::f64::< (:wat::rete::core::i64::to-f64 ?a) 0.5)`.
**Do NOT "fix" this by widening a rete op to accept mixed operands.** The explicit cast is the
feature.

### ⛔ Bucket D — the negative controls. A rider that "fixes" these has broken the thing they measure.

```
:wat::core::record?   ×2   — pinned by assert_eq! in tests/rete/probe_fence_names_the_head.rs
:wat::core::Uuid/v4   ×1   — ditto; it is PURE and only NON-DETERMINISTIC, and the message says so
probe_arc278_6b_ii_a_where_oracle_impure.wat   — exists to prove the fence REJECTS
probe_fence_names_the_head_nondet.wat          — ditto
```

## The instrument: a wat-fix codemod, NOT hand edits and NOT sed

`holon/CLAUDE.md` R21: *a structural rewrite across many `.wat` files is a **wat-fix codemod** — wat
rewriting wat.* Framework `wat/fix.wat`; recorded migrations in `wat-scripts/fixes/*.wat`; copy one
as the shape.

It is also the *correct* tool here for a reason beyond doctrine: `:wat::core::` appears all over
these files **outside** `where` forms, and those occurrences must not move. `fix-source` walks the
FORM TREE, so a rule can scope itself to the inside of `(:wat::rete::where …)`. A textual pass
cannot see that boundary. **This is why sed is not merely discouraged, it is wrong.**

## STOP triggers — rejection criteria, never permission to defer

- **STOP-1.** A head in bucket C whose operand type you cannot determine **from the declared field
  type in the same rule's LHS**. Do not guess and do not widen. Report the `file:line` and move on.
- **STOP-2.** A target name that is not in `RETE_OPS`. Every name you emit must appear as a
  `rete_name:` in `src/rete/vocabulary.rs`. If it does not, the row must be minted first — that is
  the orchestrator's call, not yours.
- **STOP-3.** Any change to a file under bucket D.
- **STOP-4.** `:derived` output changing. A spelling migration must not move a single derived fact.

## Verification you own

`./wat-scripts/perf/grid/check-where-shapes.sh` — per-family expected counts, unchanged by this work:
`boolean 15 · collection 10 · control 9 · multivar 12 · nesting 11 · numeric 10 · record 13 ·
shapes 6 · string 12`.

Run it in the **FOREGROUND** and block on it. Your turn ends when the numbers are in your hands.

## Rules of engagement

- Work only in `/home/watmin/work/holon/wat-rs/`. Any path containing `.claude/worktrees/` is
  harness state and is illegal to operate on.
- **Do NOT commit, push, stash, or revert.** The orchestrator integrates and weighs.
- **Do NOT run `cargo build` / `cargo nextest` / `cargo clippy`.** The binary is already built at
  `./target/release/wat`; use it directly. The orchestrator measures centrally, once (FM 18).
- You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
  notification is coming. Run every verification in the foreground.
